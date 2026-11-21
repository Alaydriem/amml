use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlStatus, AmmlTextureId, AmmlThread, LocatorBarIconDraw};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{SavedWaypoint, WaypointTags, WaypointWrite};

use crate::{
    ClientAdapter, EventHub, LoaderState, LocatorBarDetours, LocatorBarLayout, LocatorIconCall, ObjectRead,
    ScreenAdapter, TextureAdapter, ThreadNote, UiFlags,
};

static ADAPTER: OnceLock<LocatorBarAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);

/// `LocalPlayer* IClientInstance::getLocalPlayer() const`.
type LocalPlayerFn = unsafe extern "C" fn(usize) -> usize;

/// The `ui.locator_bar_icon` event: the player heads (or any icon) a mod draws on the locator bar in place of the
/// dot of the game.
///
/// While a mod listens, the render detour reads the waypoints of the locator bar of the local player and writes a tag
/// (the index of the waypoint) in place of the colour of each visible waypoint, calls the draw of the game, and puts
/// every colour back before it returns. The icon detour finds the waypoint of each blit by the tag it gets in the
/// colour, gives the blit the own colour of the waypoint again, raises `LocatorBarIconDraw` and draws with the edits.
/// With no listener the render detour does not tag, and a colour that is not a tag goes to the game unchanged.
///
/// The edits are `rect`, `color` and `texture`. A `texture` that the texture book does not give for drawing (unknown,
/// stale or not ready) drops every edit of that blit: the game draws its own icon in the colour of the waypoint. A
/// `rect` with a value that is not finite is not applied.
pub struct LocatorBarAdapter {
    state: &'static LoaderState,
    layout: LocatorBarLayout,
    tags: Mutex<WaypointTags>,
    // The waypoints of the walk in progress; kept to save an allocation for each frame.
    found: Mutex<Vec<SavedWaypoint>>,
    tag_thread: ThreadNote,
    icon_thread: ThreadNote,
    walk_logged: AtomicBool,
    dropped_logged: AtomicBool,
    misses: AtomicU64,
}

impl LocatorBarAdapter {
    pub const LOCATOR_BAR_ICON: &'static str = LocatorBarIconDraw::ID;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the locator bar draw and its icon blit when the catalog row resolved and the layout fits, offers
    /// `ui.locator_bar_icon` when both hooks are in, and logs one line. Works once per process: a second call hooks
    /// nothing and gives the catalog row.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.id == Self::LOCATOR_BAR_ICON)
            .collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the locator bar adapter is already installed");
            return rows;
        }
        let shaped = rows.iter().any(|row| row.shape.is_some());
        let layout = shaped.then(|| LocatorBarLayout::from_state(state)).flatten();
        // The icon hook goes first: with only the render hook in, the tags would reach the draw of the game.
        let on = layout.is_some()
            && LocatorBarDetours::draw_icon_hook().install(state, Self::PRIORITY)
            && LocatorBarDetours::render_hook().install(state, Self::PRIORITY);
        if let (true, Some(layout)) = (on, layout) {
            let adapter = Self {
                state,
                layout,
                tags: Mutex::new(WaypointTags::new()),
                found: Mutex::new(Vec::with_capacity(WaypointTags::CAPACITY)),
                tag_thread: ThreadNote::new(),
                icon_thread: ThreadNote::new(),
                walk_logged: AtomicBool::new(false),
                dropped_logged: AtomicBool::new(false),
                misses: AtomicU64::new(0),
            };
            if ADAPTER.set(adapter).is_err() {
                curia::warn!("the locator bar adapter is already installed");
                return rows;
            }
        }
        for row in &mut rows {
            if row.shape.is_some() && !on {
                row.shape = None;
                row.missing = vec!["a hook or a layout"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Event) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                _ => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`, and when the locator bar is not served.
    pub fn get() -> Option<&'static LocatorBarAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` when there is none.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&LocatorBarAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Before the game draws the locator bar of `client`: tags the colour of every visible waypoint and sets
    /// `UiFlags::LOCATOR_TAGGED`. Does nothing when the client has no local player, the list does not look sane or
    /// holds more waypoints than a frame tags.
    pub(crate) fn tag(&self, client: usize) {
        let game = self.state.events().gate().on_game_thread();
        if self.tag_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("ui thread", { hook: LocatorBarDetours::RENDER, game: game });
        }
        let Some(head) = self.list_head(client) else { return };
        let mut found = Self::lock(&self.found);
        found.clear();
        // SAFETY: `head` is the sentinel of the live waypoint list of the local player, which only the thread of the
        // draw changes while the draw runs.
        if !unsafe { self.walk(head, &mut found) } {
            return;
        }
        let layout = self.layout;
        let mut tags = Self::lock(&self.tags);
        let tagged = tags.tag(found.iter().copied(), |waypoint, write| match write {
            WaypointWrite::Tag(tag) => {
                // SAFETY: `waypoint` was read live by `walk`; its colour and the engaged byte are inside it, and the
                // layout checked that the engaged byte lies inside the saved colour bytes.
                unsafe {
                    std::ptr::with_exposed_provenance_mut::<[f32; 4]>(waypoint + layout.color).write_unaligned(tag);
                    std::ptr::with_exposed_provenance_mut::<u8>(waypoint + layout.color_engaged).write(1);
                }
            }
            WaypointWrite::Saved(bytes) => {
                // SAFETY: the waypoint was tagged by an earlier `tag` and the game keeps it alive; the saved bytes
                // are the colour member of it.
                unsafe {
                    std::ptr::with_exposed_provenance_mut::<[u8; SavedWaypoint::COLOR_BYTES]>(waypoint + layout.color)
                        .write_unaligned(bytes);
                }
            }
        });
        if tagged {
            UiFlags::set(UiFlags::LOCATOR_TAGGED);
        }
        if !self.walk_logged.swap(true, Ordering::Relaxed) {
            let visible = found.len();
            curia::info!("locator bar first walk", {
                node_next_offset: LocatorBarLayout::NODE_NEXT,
                visible: visible,
                tagged: tagged
            });
        }
    }

    /// After the game drew the locator bar: puts the saved colours back and clears `UiFlags::LOCATOR_TAGGED`.
    pub(crate) fn restore(&self) {
        let layout = self.layout;
        Self::lock(&self.tags).restore(|waypoint, bytes| {
            // SAFETY: the waypoint was tagged by `tag` in this draw and the game keeps it alive until the draw ends;
            // the saved bytes are the colour member of it.
            unsafe {
                std::ptr::with_exposed_provenance_mut::<[u8; SavedWaypoint::COLOR_BYTES]>(waypoint + layout.color)
                    .write_unaligned(*bytes);
            }
        });
        UiFlags::clear(UiFlags::LOCATOR_TAGGED);
    }

    // The sentinel of the waypoint list of the local player of `client`; `None` without a client, a local player or a
    // list.
    fn list_head(&self, client: usize) -> Option<usize> {
        let function = ClientAdapter::virtual_fn(client, Some(self.layout.local_player_slot))?;
        // SAFETY: the slot holds `LocalPlayer* IClientInstance::getLocalPlayer() const`, called on the client the
        // game gave its draw.
        let player = unsafe { std::mem::transmute::<usize, LocalPlayerFn>(function)(client) };
        // SAFETY: a non-null player is live for the draw; the list pointer is inside its `ClientLocatorBar`.
        unsafe { ObjectRead::pointer(player, self.layout.bar.checked_add(self.layout.list_head)?) }
    }

    // Collects the visible waypoints of the list with the sentinel `head` into `found`. False when a node or a read
    // is not usable, the list is longer than `MAX_NODES`, or more waypoints are visible than a frame tags.
    //
    // Safety: `head` is the sentinel node of a live, circular list whose nodes hold waypoints that stay alive for
    // the call.
    unsafe fn walk(&self, head: usize, found: &mut Vec<SavedWaypoint>) -> bool {
        let layout = &self.layout;
        // SAFETY: the sentinel is a live node; the next pointer is its first member.
        let Some(mut node) = (unsafe { ObjectRead::pointer(head, LocatorBarLayout::NODE_NEXT) }) else { return false };
        let mut walked = 0;
        while node != head {
            walked += 1;
            if walked > LocatorBarLayout::MAX_NODES {
                return false;
            }
            // SAFETY: `node` is a live list node holding a next pointer and a `unique_ptr<ClientWaypoint>`.
            let next = unsafe { ObjectRead::pointer(node, LocatorBarLayout::NODE_NEXT) };
            // SAFETY: as above.
            let waypoint = unsafe { ObjectRead::value::<usize>(node, layout.node_waypoint) };
            let Some(next) = next else { return false };
            if let Some(waypoint) = waypoint.filter(|waypoint| *waypoint != 0) {
                // SAFETY: the node holds a live waypoint; the visible flag is inside it.
                let visible = unsafe { ObjectRead::value::<u8>(waypoint, layout.visible) };
                if visible.is_some_and(|visible| visible != 0) {
                    // SAFETY: as above; the colour bytes are inside the waypoint.
                    let Some(color) =
                        (unsafe { ObjectRead::value::<[u8; SavedWaypoint::COLOR_BYTES]>(waypoint, layout.color) })
                    else {
                        return false;
                    };
                    // SAFETY: the waypoint is live; the vtable check guards the read of the actor id.
                    let actor = unsafe { self.actor_of(waypoint) };
                    found.push(SavedWaypoint::new(waypoint, color, actor));
                    // A frame holds at most `CAPACITY`; one more is enough to make `tag` refuse.
                    if found.len() > WaypointTags::CAPACITY {
                        return false;
                    }
                }
            }
            node = next;
        }
        true
    }

    // The actor of a waypoint that is a `ClientEntityWaypoint`; `None` for another kind.
    //
    // Safety: `waypoint` is a live `ClientWaypoint`.
    unsafe fn actor_of(&self, waypoint: usize) -> Option<i64> {
        // SAFETY: a waypoint starts with its vtable pointer.
        let vtable = unsafe { ObjectRead::value::<usize>(waypoint, 0) }?;
        if vtable != self.layout.entity_vtable {
            return None;
        }
        // SAFETY: the vtable matched, so the object is a `ClientEntityWaypoint` that holds the id at `actor_id`.
        unsafe { ObjectRead::value::<i64>(waypoint, self.layout.actor_id) }
    }

    /// One blit of the icon of `call`: finds the waypoint by the tag in the colour, raises `LocatorBarIconDraw`
    /// while a mod listens, and gives the call to draw. `None` when the colour is not a tag of this frame: the game
    /// then draws the call it made.
    pub(crate) fn icon(&self, call: LocatorIconCall) -> Option<LocatorIconCall> {
        let saved = Self::lock(&self.tags).lookup(call.color).copied();
        let Some(saved) = saved else {
            let misses = self.misses.fetch_add(1, Ordering::Relaxed) + 1;
            if misses.is_power_of_two() {
                curia::warn!("a colour in a tagged frame did not decode", { misses: misses });
            }
            return None;
        };
        let real = saved.vanilla_packed();
        let mut draw = LocatorIconCall { color: real, ..call };
        let hub = self.state.events();
        if !hub.has_listeners(LocatorBarIconDraw::ID) {
            return Some(draw);
        }
        let context = ScreenAdapter::context_of(self.state, None);
        if self.icon_thread.changed(context.thread) {
            curia::info!("ui thread", { hook: LocatorBarDetours::DRAW_ICON, game: context.thread == AmmlThread::GAME });
        }
        let actor = saved.actor();
        let sent = LocatorBarIconDraw {
            header: EventHub::header::<LocatorBarIconDraw>(AmmlPhase::BEFORE),
            waypoint_actor_id: actor.unwrap_or(0),
            has_actor: u8::from(actor.is_some()),
            _reserved: [0; 7],
            rect: call.rect,
            uv: call.uv,
            light: call.light,
            alpha: call.alpha,
            color: real,
            second_color: call.second_color,
            scale: call.scale,
            texture: AmmlTextureId::default(),
        };
        let mut event = sent;
        hub.raise_keeping(&mut event, &context, &|value| {
            value.waypoint_actor_id = sent.waypoint_actor_id;
            value.has_actor = sent.has_actor;
            value._reserved = sent._reserved;
            value.uv = sent.uv;
            value.light = sent.light;
            value.alpha = sent.alpha;
            value.second_color = sent.second_color;
            value.scale = sent.scale;
        });
        if event.texture.id != 0 {
            let key = TextureAdapter::key_of(event.texture);
            match TextureAdapter::with(Err(AmmlStatus::NOT_AVAILABLE), |adapter| adapter.drawable(key)) {
                Ok(Some(texture)) => draw.texture = texture,
                _ => {
                    if !self.dropped_logged.swap(true, Ordering::Relaxed) {
                        curia::info!("locator icon edits dropped", { reason: "texture not drawable" });
                    }
                    return Some(draw);
                }
            }
        }
        if event.rect.iter().all(|value| value.is_finite()) {
            draw.rect = event.rect;
        }
        draw.color = event.color;
        Some(draw)
    }
}

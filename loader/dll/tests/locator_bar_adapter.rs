use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use amml::{LoaderState, LocatorBarAdapter, ModListener, UiFlags};
use amml_abi::{AmmlBand, AmmlEventContext, AmmlEventHeader, AmmlStatus, AmmlTextureId, LocatorBarIconDraw};
use amml_data::{EntryKind, Resolved};
use amml_runtime::WaypointTags;

use crate::support::{CLIENT_A, Harness, SessionGame};

// The offsets of the stand-in player, locator bar, list node and waypoints.
const BAR_AT: usize = 0x40;
const LIST_HEAD_AT: usize = 0x10;
const NODE_WAYPOINT_AT: usize = 0x10;
const COLOR_AT: usize = 0x18;
const ENGAGED_AT: usize = 0x28;
const VISIBLE_AT: usize = 0x30;
const ACTOR_AT: usize = 0x78;
const WORDS: usize = 24;

type RenderFn = extern "C" fn(usize, usize, usize, usize, i32);
type DrawFn = extern "C" fn(usize, usize, usize, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, u32, u32, f32, f32);

// The vtable of the stand-in renderer: the render is slot 1.
static RENDERER_VTABLE: [RenderFn; 3] = [unused, render_stand_in, unused];
// The vtables of the stand-in waypoints; the loader tells an entity waypoint by the first one.
static ENTITY_VTABLE: [u64; 4] = [0; 4];
static PLAIN_VTABLE: [u64; 4] = [0; 4];

// The waypoints the stand-in game draws, as addresses, and what it did.
static WAYPOINTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
// The colour of each visible waypoint as the game read it when it started to draw.
static READ_COLORS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
// One icon blit as the game's function received it.
type Blit = (usize, [f32; 4], [f32; 4], f32, f32, u32, u32, [f32; 2]);
static BLITS: Mutex<Vec<Blit>> = Mutex::new(Vec::new());
// What the stand-in game does in the middle of its draw, after the colours were read and before the icons are blit.
static DURING: Mutex<Option<fn()>> = Mutex::new(None);

#[inline(never)]
extern "C" fn unused(_this: usize, _context: usize, _client: usize, _owner: usize, _pass: i32) {
    black_box(0);
}

#[inline(never)]
extern "C" fn render_stand_in(this: usize, context: usize, client: usize, owner: usize, pass: i32) {
    black_box((client, owner, pass));
    let waypoints = WAYPOINTS.lock().unwrap().clone();
    let during = *DURING.lock().unwrap();
    if let Some(during) = during {
        during();
    }
    for waypoint in waypoints {
        if read::<u8>(waypoint, VISIBLE_AT) == 0 {
            continue;
        }
        let packed = if read::<u8>(waypoint, ENGAGED_AT) == 0 {
            0xFFFF_FFFF
        } else {
            WaypointTags::pack(read::<[f32; 4]>(waypoint, COLOR_AT))
        };
        READ_COLORS.lock().unwrap().push(packed);
        let draw: DrawFn = black_box(draw_icon_stand_in);
        draw(this, context, 0x77, 10.0, 20.0, 0.0, 1.0, 0.0, 1.0, 8.0, 9.0, 0.5, 0.75, packed, 0xFF00_0000, 1.0, 2.0);
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
extern "C" fn draw_icon_stand_in(
    this: usize,
    context: usize,
    texture: usize,
    x: f32,
    y: f32,
    u0: f32,
    u1: f32,
    v0: f32,
    v1: f32,
    width: f32,
    height: f32,
    light: f32,
    alpha: f32,
    color: u32,
    second_color: u32,
    scale_x: f32,
    scale_y: f32,
) {
    black_box((this, context));
    BLITS.lock().unwrap().push((
        texture,
        [x, y, width, height],
        [u0, u1, v0, v1],
        light,
        alpha,
        black_box(color),
        second_color,
        [scale_x, scale_y],
    ));
}

fn read<T: Copy>(base: usize, offset: usize) -> T {
    // SAFETY: the stand-in memory is leaked, at least `WORDS` words, and every read is inside it.
    unsafe { std::ptr::with_exposed_provenance::<T>(base + offset).read_unaligned() }
}

fn write<T: Copy>(base: usize, offset: usize, value: T) {
    // SAFETY: as for `read`.
    unsafe { std::ptr::with_exposed_provenance_mut::<T>(base + offset).write_unaligned(value) };
}

fn leak() -> usize {
    Box::leak(Box::new([0u64; WORDS])).as_mut_ptr().expose_provenance()
}

fn entries() -> Vec<(&'static str, Resolved)> {
    let rva = |address: usize| address as u64 - Harness::BASE;
    let global = |address: usize| Resolved { kind: EntryKind::Global, value: rva(address), recipes_agreeing: 2 };
    let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
    vec![
        ("ClientInstance.getLocalPlayer", Resolved { kind: EntryKind::VSlot, value: 0, recipes_agreeing: 2 }),
        ("HudLocatorBarRenderer.vtable", global(std::ptr::from_ref(&RENDERER_VTABLE) as usize)),
        ("HudLocatorBarRenderer.render", Resolved { kind: EntryKind::VSlot, value: 1, recipes_agreeing: 2 }),
        (
            "HudLocatorBarRenderer.drawIcon",
            Resolved {
                kind: EntryKind::Function,
                value: rva(draw_icon_stand_in as *const () as usize),
                recipes_agreeing: 2,
            },
        ),
        ("LocalPlayer.mClientLocatorBar", field(BAR_AT)),
        ("ClientLocatorBar.listHead", field(LIST_HEAD_AT)),
        ("ClientLocatorBar.nodeWaypoint", field(NODE_WAYPOINT_AT)),
        ("ClientWaypoint.mColor", field(COLOR_AT)),
        ("ClientWaypoint.mColorEngaged", field(ENGAGED_AT)),
        ("ClientWaypoint.mVisible", field(VISIBLE_AT)),
        ("ClientEntityWaypoint.vtable", global(std::ptr::from_ref(&ENTITY_VTABLE) as usize)),
        ("ClientEntityWaypoint.mActorId", field(ACTOR_AT)),
    ]
}

fn start() -> Harness {
    let harness = Harness::start_with_entries(&entries());
    let rows = LocatorBarAdapter::install(harness.state);
    assert!(rows.iter().all(|row| row.shape.is_some()), "{rows:?}");
    harness
}

// The colour bytes of a waypoint: four floats, the engaged byte and padding.
fn color_bytes(lanes: [f32; 4], engaged: u8, padding: u8) -> [u8; 20] {
    let mut bytes = [padding; 20];
    for (index, lane) in lanes.iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&lane.to_le_bytes());
    }
    bytes[16] = engaged;
    bytes
}

// What one stand-in waypoint is: the actor it follows (`None` for a plain waypoint), whether it is visible and its
// colour bytes.
struct Spec {
    actor: Option<i64>,
    visible: bool,
    color: [u8; 20],
}

// A stand-in local player with a locator bar that holds `specs`, and the waypoint addresses. The player is the one the
// stand-in client gives.
fn world(specs: &[Spec]) -> Vec<usize> {
    let player = leak();
    let sentinel = leak();
    write(player, BAR_AT + LIST_HEAD_AT, sentinel);
    let mut previous = sentinel;
    let mut waypoints = Vec::new();
    for spec in specs {
        let waypoint = leak();
        let vtable = if spec.actor.is_some() { &ENTITY_VTABLE } else { &PLAIN_VTABLE };
        write(waypoint, 0, std::ptr::from_ref(vtable).expose_provenance());
        write(waypoint, COLOR_AT, spec.color);
        write(waypoint, VISIBLE_AT, u8::from(spec.visible));
        write(waypoint, ACTOR_AT, spec.actor.unwrap_or(0));
        let node = leak();
        write(node, NODE_WAYPOINT_AT, waypoint);
        write(previous, 0, node);
        previous = node;
        waypoints.push(waypoint);
    }
    write(previous, 0, sentinel);
    SessionGame::set_player(player);
    *WAYPOINTS.lock().unwrap() = waypoints.clone();
    READ_COLORS.lock().unwrap().clear();
    BLITS.lock().unwrap().clear();
    waypoints
}

fn render() {
    let function: RenderFn = black_box(RENDERER_VTABLE[1]);
    function(0x10, 0x20, SessionGame::client(&CLIENT_A).expose_provenance(), 0x30, 0);
}

fn three() -> Vec<Spec> {
    vec![
        Spec { actor: Some(100), visible: true, color: color_bytes([0.2, 0.4, 0.6, 1.0], 1, 0x11) },
        Spec { actor: None, visible: true, color: color_bytes([0.0, 0.0, 0.0, 0.0], 0, 0x22) },
        Spec { actor: Some(102), visible: false, color: color_bytes([1.0, 0.0, 0.0, 1.0], 1, 0x33) },
    ]
}

fn colors_of(waypoints: &[usize]) -> Vec<[u8; 20]> {
    waypoints.iter().map(|waypoint| read::<[u8; 20]>(*waypoint, COLOR_AT)).collect()
}

// What the icon listener does: 0 reads, 1 changes rect and colour, 2 asks for a texture the book does not have, 3
// changes the fields that are the loader's as well as the colour, 4 asks for the plain texture id 0.
static ACTION: AtomicU8 = AtomicU8::new(0);
// What the listener saw: the actor, the colour, the rect and the uv.
type Seen = (Option<i64>, u32, [f32; 4], [f32; 4], f32);
static SEEN: Mutex<Vec<Seen>> = Mutex::new(Vec::new());

unsafe extern "C" fn on_icon(event: *mut AmmlEventHeader, _: *const AmmlEventContext, _: *mut c_void) -> AmmlStatus {
    // SAFETY: the loader raises `ui.locator_bar_icon` with a `LocatorBarIconDraw`.
    let event = unsafe { &mut *event.cast::<LocatorBarIconDraw>() };
    SEEN.lock().unwrap().push((
        (event.has_actor != 0).then_some(event.waypoint_actor_id),
        event.color,
        event.rect,
        event.uv,
        event.light,
    ));
    match ACTION.load(Ordering::SeqCst) {
        1 => {
            event.rect = [1.0, 2.0, 3.0, 4.0];
            event.color = 0xFF11_2233;
        }
        2 => {
            event.rect = [1.0, 2.0, 3.0, 4.0];
            event.color = 0xFF11_2233;
            event.texture = AmmlTextureId { id: 999, generation: 0, _reserved: 0 };
        }
        3 => {
            event.uv = [9.0; 4];
            event.light = 9.0;
            event.alpha = 9.0;
            event.second_color = 9;
            event.scale = [9.0; 2];
            event.waypoint_actor_id = 9;
            event.has_actor = 0;
            event.color = 0xFF44_5566;
        }
        4 => {
            event.rect = [5.0, 6.0, f32::NAN, 8.0];
            event.texture = AmmlTextureId::default();
        }
        _ => {}
    }
    AmmlStatus::OK
}

fn listen(harness: &Harness) -> amml_runtime::ListenerId {
    let listener = ModListener { owner: 0, callback: on_icon, user: 0 };
    harness.state.events().listen("ui.locator_bar_icon", 1, AmmlBand::NORMAL, listener).unwrap()
}

fn state_of(harness: &Harness) -> &'static LoaderState {
    harness.state
}

#[test]
fn with_no_listener_the_game_draws_untouched_and_no_colour_is_tagged() {
    let harness = start();
    let waypoints = world(&three());
    let before = colors_of(&waypoints);
    assert_eq!(UiFlags::load(), 0);

    render();

    assert_eq!(colors_of(&waypoints), before);
    let plain = WaypointTags::pack([0.2, 0.4, 0.6, 1.0]);
    assert_eq!(*READ_COLORS.lock().unwrap(), [plain, 0xFFFF_FFFF]);
    let blits = BLITS.lock().unwrap();
    assert_eq!(blits.iter().map(|blit| blit.5).collect::<Vec<_>>(), [plain, 0xFFFF_FFFF]);
    assert_eq!(
        blits[0],
        (0x77, [10.0, 20.0, 8.0, 9.0], [0.0, 1.0, 0.0, 1.0], 0.5, 0.75, plain, 0xFF00_0000, [1.0, 2.0])
    );
    assert!(!state_of(&harness).events().has_listeners("ui.locator_bar_icon"));
}

#[test]
fn a_listener_sees_each_visible_waypoint_with_its_own_colour_and_the_game_reads_tags() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    SEEN.lock().unwrap().clear();
    let waypoints = world(&three());
    let before = colors_of(&waypoints);
    listen(&harness);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);

    render();

    // The game read tags, in list order, for the two visible waypoints only.
    let read = READ_COLORS.lock().unwrap().clone();
    assert_eq!(read, [WaypointTags::pack(WaypointTags::encode(0)), WaypointTags::pack(WaypointTags::encode(1))]);
    // The listener saw the waypoint's own colour (white for an empty colour) and who it follows.
    let plain = WaypointTags::pack([0.2, 0.4, 0.6, 1.0]);
    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    assert_eq!((seen[0].0, seen[0].1, seen[0].2), (Some(100), plain, [10.0, 20.0, 8.0, 9.0]));
    assert_eq!((seen[1].0, seen[1].1), (None, 0xFFFF_FFFF));
    // The game's draw got the waypoint's own colours, not the tags.
    let blits = BLITS.lock().unwrap();
    assert_eq!(blits.iter().map(|blit| blit.5).collect::<Vec<_>>(), [plain, 0xFFFF_FFFF]);
    // Every colour is back, byte for byte, including the hidden waypoint's and the empty one's padding.
    assert_eq!(colors_of(&waypoints), before);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);
}

#[test]
fn rect_and_colour_edits_reach_the_game_and_uv_light_scale_and_actor_do_not_change() {
    let harness = start();
    ACTION.store(1, Ordering::SeqCst);
    SEEN.lock().unwrap().clear();
    world(&three());
    listen(&harness);

    render();

    let blits = BLITS.lock().unwrap();
    assert_eq!(blits.len(), 2);
    for blit in blits.iter() {
        assert_eq!((blit.0, blit.1, blit.5), (0x77, [1.0, 2.0, 3.0, 4.0], 0xFF11_2233));
        assert_eq!(
            (blit.2, blit.3, blit.4, blit.6, blit.7),
            ([0.0, 1.0, 0.0, 1.0], 0.5, 0.75, 0xFF00_0000, [1.0, 2.0])
        );
    }
}

#[test]
fn a_listener_cannot_change_the_fields_that_are_the_loaders_for_the_next_listener_or_the_game() {
    let harness = start();
    ACTION.store(3, Ordering::SeqCst);
    SEEN.lock().unwrap().clear();
    world(&three());
    listen(&harness);
    listen(&harness);

    render();

    let seen = SEEN.lock().unwrap().clone();
    // Four calls: two listeners for each of two waypoints. The second listener of a waypoint saw the loader's fields.
    assert_eq!(seen.len(), 4);
    assert_eq!((seen[1].0, seen[1].3, seen[1].4), (Some(100), [0.0, 1.0, 0.0, 1.0], 0.5));
    let blits = BLITS.lock().unwrap();
    assert_eq!(
        (blits[0].2, blits[0].3, blits[0].4, blits[0].6, blits[0].7),
        ([0.0, 1.0, 0.0, 1.0], 0.5, 0.75, 0xFF00_0000, [1.0, 2.0])
    );
    assert_eq!(blits[0].5, 0xFF44_5566);
}

#[test]
fn a_texture_the_book_does_not_give_drops_every_edit() {
    let harness = start();
    ACTION.store(2, Ordering::SeqCst);
    world(&three());
    listen(&harness);

    render();

    let plain = WaypointTags::pack([0.2, 0.4, 0.6, 1.0]);
    let blits = BLITS.lock().unwrap();
    assert_eq!(blits.len(), 2);
    // The game's own icon: its texture, its rect, and the colour of the waypoint.
    assert_eq!((blits[0].0, blits[0].1, blits[0].5), (0x77, [10.0, 20.0, 8.0, 9.0], plain));
    assert_eq!((blits[1].0, blits[1].1, blits[1].5), (0x77, [10.0, 20.0, 8.0, 9.0], 0xFFFF_FFFF));
}

#[test]
fn a_rect_that_is_not_finite_is_not_applied_and_texture_zero_keeps_the_icon() {
    let harness = start();
    ACTION.store(4, Ordering::SeqCst);
    world(&three());
    listen(&harness);

    render();

    let blits = BLITS.lock().unwrap();
    assert_eq!((blits[0].0, blits[0].1), (0x77, [10.0, 20.0, 8.0, 9.0]));
}

#[test]
fn a_list_that_never_reaches_its_sentinel_tags_nothing() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    let waypoints = world(&three());
    let before = colors_of(&waypoints);
    // A player whose list holds one node that points to itself.
    let looping = leak();
    write(looping, 0, looping);
    write(looping, NODE_WAYPOINT_AT, waypoints[0]);
    let sentinel = leak();
    write(sentinel, 0, looping);
    let player = leak();
    write(player, BAR_AT + LIST_HEAD_AT, sentinel);
    SessionGame::set_player(player);
    listen(&harness);

    render();

    assert_eq!(colors_of(&waypoints), before);
    let plain = WaypointTags::pack([0.2, 0.4, 0.6, 1.0]);
    assert_eq!(BLITS.lock().unwrap().iter().map(|blit| blit.5).collect::<Vec<_>>(), [plain, 0xFFFF_FFFF]);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);
}

#[test]
fn without_a_local_player_nothing_is_tagged() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    let waypoints = world(&three());
    let before = colors_of(&waypoints);
    SessionGame::set_player(0);
    listen(&harness);

    render();

    assert_eq!(colors_of(&waypoints), before);
    assert_eq!(BLITS.lock().unwrap().len(), 2);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);
}

#[test]
fn the_flag_follows_the_listeners() {
    let harness = start();
    assert_eq!(UiFlags::load(), 0);

    let first = listen(&harness);
    let second = listen(&harness);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);
    harness.state.events().unlisten(first);
    assert_eq!(UiFlags::load(), UiFlags::LOCATOR);
    harness.state.events().unlisten(second);

    assert_eq!(UiFlags::load(), 0);
}

static LISTENER: Mutex<Option<(&'static LoaderState, amml_runtime::ListenerId)>> = Mutex::new(None);

fn unlisten_now() {
    let taken = LISTENER.lock().unwrap().take();
    if let Some((state, id)) = taken {
        state.events().unlisten(id);
    }
}

#[test]
fn a_listener_that_leaves_during_the_draw_leaves_no_tag_behind() {
    let harness = start();
    ACTION.store(0, Ordering::SeqCst);
    let waypoints = world(&three());
    let before = colors_of(&waypoints);
    let id = listen(&harness);
    *LISTENER.lock().unwrap() = Some((harness.state, id));
    *DURING.lock().unwrap() = Some(unlisten_now);

    render();

    // The game read tags, the listener went away, and the colours still came back; the icons got their own colours.
    assert_eq!(READ_COLORS.lock().unwrap().len(), 2);
    assert_eq!(colors_of(&waypoints), before);
    assert_eq!(UiFlags::load(), 0);
    let plain = WaypointTags::pack([0.2, 0.4, 0.6, 1.0]);
    assert_eq!(BLITS.lock().unwrap().iter().map(|blit| blit.5).collect::<Vec<_>>(), [plain, 0xFFFF_FFFF]);
}

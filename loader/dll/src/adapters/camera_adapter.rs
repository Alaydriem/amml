use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use amml_abi::{AmmlEvent, AmmlPhase, AmmlPose, AmmlStatus, AmmlThread, CameraSetup};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{CameraMath, CameraScope, PoseBook};

use crate::{
    CameraDetours, CameraLayoutError, CameraProjection, CameraWriter, EventHub, LoaderState, SessionAdapter, ThreadNote,
};

static ADAPTER: OnceLock<CameraAdapter> = OnceLock::new();
// The count of `FrameUpdate` Before events, the `frame_id` of `CameraSetup`.
static FRAMES: AtomicU64 = AtomicU64::new(0);

thread_local! {
    // The camera of the `setupCamera` call whose `CameraSetup` dispatch runs on this thread, and whether the dispatch is
    // the After one; zero outside a dispatch.
    static CALL: Cell<(usize, bool)> = const { Cell::new((0, false)) };
}

/// The camera adapter: raises `CameraSetup` Before and After the game's `setupCamera`, answers the camera calls of the
/// dispatch and writes the pose a mod set (`PoseBook`) into the game's camera, view object and level camera.
///
/// `frame_id` is the count of `FrameUpdate` Before events; a camera call names the `frame_id` of the dispatch
/// (`CameraScope`), and any other frame or a call from another thread is `REFUSED`. A pose is for its frame only: the
/// loader writes the rotation after the game's `setupCamera` and the After listeners, the view position after
/// `createViewRenderObject` and the level camera position after `preRenderUpdate`. A frame without a pose writes
/// nothing. A disabled mod's pose is dropped at once; a world leave forgets the pose and the rendered position.
///
/// The event is raised on the thread that runs `setupCamera` (logged as `camera setup thread` once and when it
/// changes). A game call from a `CameraSetup` listener on another thread than the game thread is `REFUSED`
/// (`check_game_call`), logged once as `camera setup off the game thread`.
pub struct CameraAdapter {
    state: &'static LoaderState,
    poses: PoseBook,
    writer: Option<CameraWriter>,
    projection: Option<CameraProjection>,
    thread: ThreadNote,
    off_thread_logged: AtomicBool,
}

impl CameraAdapter {
    pub const POSE: &'static str = "camera.pose";
    pub const PROJECTION: &'static str = "camera.projection";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    const VERSION: u32 = 1;

    /// Hooks `setupCamera` when the `camera.setup` row resolved, and the two position functions when `camera.pose`
    /// resolved and its mirrors fit their entries; offers `camera.setup`, `camera.pose` and `camera.projection`. An area
    /// is offered only with the `setupCamera` hook, and only when its mirrors fit (else `camera layout mismatch` names
    /// the entry). Logs one `event offered` / `area adapter` or `... not available` line per row. Works once per
    /// process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let lookup = |name: &str| state.resolve(name).ok();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let setup_resolved = rows.iter().any(|row| row.id == CameraSetup::ID && row.shape.is_some());
        let hooked = setup_resolved && CameraDetours::setup_hook().install(state, Self::PRIORITY);
        let mut writer = None;
        let mut projection = None;
        for row in &mut rows {
            if row.shape.is_none() {
                continue;
            }
            if !hooked {
                row.shape = None;
                row.missing = vec![CameraDetours::SETUP_CAMERA];
                continue;
            }
            let checked = match row.id {
                Self::POSE => CameraWriter::from_lookup(&lookup).map(|found| writer = Some(found)),
                Self::PROJECTION => CameraProjection::from_lookup(&lookup).map(|found| projection = Some(found)),
                _ => Ok(()),
            };
            if let Err(error) = checked {
                Self::refuse_row(row, error);
                continue;
            }
            if row.id == Self::POSE {
                let failed = CameraDetours::position_hooks()
                    .into_iter()
                    .find(|hook| !hook.install(state, Self::PRIORITY))
                    .map(|hook| hook.entry);
                if let Some(entry) = failed {
                    writer = None;
                    row.shape = None;
                    row.missing = vec![entry];
                }
            }
        }
        let adapter = Self {
            state,
            poses: PoseBook::new(),
            writer,
            projection,
            thread: ThreadNote::new(),
            off_thread_logged: AtomicBool::new(false),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the camera adapter is already installed");
            return rows;
        }
        for row in &rows {
            match (row.kind, row.shape) {
                (NeedKind::Event, Some(shape)) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (NeedKind::Area, Some(shape)) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (NeedKind::Event, None) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
                (NeedKind::Area, None) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        rows
    }

    fn refuse_row(row: &mut Availability, error: CameraLayoutError) {
        curia::warn!("camera layout mismatch", { area: row.id, entry: error.entry(), detail: error.to_string() });
        row.shape = None;
        row.missing = vec![error.entry()];
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static CameraAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&CameraAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => row.id == CameraSetup::ID,
            NeedKind::Area => row.id == Self::POSE || row.id == Self::PROJECTION,
        }
    }

    /// A `FrameUpdate` Before: the next `CameraSetup` has a new `frame_id`. Game thread.
    pub(crate) fn frame_began() {
        FRAMES.fetch_add(1, Ordering::Relaxed);
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    fn check_area(&self, area: &str) -> Result<(), AmmlStatus> {
        if self.state.areas().is_available(area, Self::VERSION) { Ok(()) } else { Err(AmmlStatus::NOT_AVAILABLE) }
    }

    fn note_pose(&self) {
        CameraDetours::set(CameraDetours::POSE, self.poses.is_held());
    }

    /// Before the game's `setupCamera` of `camera`, on its thread: starts the camera frame and raises `CameraSetup`
    /// Before. Gives the frame id and whether this is the first setup of the frame, or `None` while no world is joined
    /// or before the first `FrameUpdate`.
    pub(crate) fn before_setup(&self, camera: usize) -> Option<(u64, bool)> {
        let gate = self.hub().gate();
        if !gate.is_open() || gate.epoch().0 == 0 {
            return None;
        }
        let frame = FRAMES.load(Ordering::Relaxed);
        if frame == 0 {
            return None;
        }
        let first = self.poses.begin_frame(frame);
        self.note_pose();
        let game = gate.on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("camera setup thread", { game: game });
        }
        self.raise(AmmlPhase::BEFORE, frame, first, camera);
        Some((frame, first))
    }

    /// After the game's `setupCamera` of `camera`, on its thread: raises `CameraSetup` After, then writes the rotation of
    /// the pose of the frame.
    pub(crate) fn after_setup(&self, (frame, first): (u64, bool), camera: usize) {
        self.raise(AmmlPhase::AFTER, frame, first, camera);
        self.note_pose();
        let (Some(writer), Some(pose)) = (&self.writer, self.poses.pose()) else { return };
        if self.check_area(Self::POSE).is_err() {
            return;
        }
        let basis = CameraMath::basis(pose.rotation, pose.roll);
        // SAFETY: `camera` is the camera of the `setupCamera` call that just returned on this thread, live for it.
        unsafe { writer.write_rotation(std::ptr::with_exposed_provenance_mut(camera), &basis) };
    }

    /// After the game's `createViewRenderObject` filled `view_object`: writes the position of the pose of the frame.
    pub(crate) fn after_view_object(&self, view_object: usize) {
        let Some(writer) = &self.writer else { return };
        let Some(position) = self.poses.write_position() else { return };
        // SAFETY: `view_object` is the out object the game's call just filled, live for the caller.
        unsafe { writer.write_view_position(view_object, [position.x, position.y, position.z]) };
    }

    /// After the game's `preRenderUpdate` of `level_camera`: writes the position of the pose of the frame for the chunk
    /// sort.
    pub(crate) fn after_pre_render(&self, level_camera: usize) {
        let Some(writer) = &self.writer else { return };
        let Some(position) = self.poses.write_position() else { return };
        // SAFETY: `level_camera` is the `this` of the call that just returned, a live `LevelRendererCamera`.
        unsafe { writer.write_level_camera_position(level_camera, [position.x, position.y, position.z]) };
    }

    // Raises `CameraSetup` in `phase` while a mod listens; no listener changes the frame id, `first_in_frame` or the
    // reserved bytes.
    fn raise(&self, phase: AmmlPhase, frame: u64, first: bool, camera: usize) {
        let hub = self.hub();
        if !hub.has_listeners(CameraSetup::ID) {
            return;
        }
        let game = hub.gate().on_game_thread();
        let first_in_frame = u8::from(first);
        let mut event = CameraSetup {
            header: EventHub::header::<CameraSetup>(phase),
            frame_id: frame,
            first_in_frame,
            _reserved: [0; 7],
        };
        let mut context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::RENDER });
        if game && let Some(client) = SessionAdapter::primary_client() {
            context.client = std::ptr::with_exposed_provenance(client);
        }
        let keep = |event: &mut CameraSetup| {
            event.frame_id = frame;
            event.first_in_frame = first_in_frame;
            event._reserved = [0; 7];
        };
        let _frame = CameraScope::enter(frame);
        let previous = CALL.with(|cell| cell.replace((camera, phase == AmmlPhase::AFTER)));
        hub.raise_keeping(&mut event, &context, &keep);
        CALL.with(|cell| cell.set(previous));
    }

    /// Sets the pose of `frame` for the mod at `owner`. `NOT_AVAILABLE` without `camera.pose`, `REFUSED` outside the
    /// dispatch of `frame` on this thread or when another mod set the pose of the frame first, `INVALID_ARGUMENT` for a
    /// pose that is not finite.
    pub fn set_pose(&self, owner: usize, frame: u64, pose: AmmlPose) -> Result<(), AmmlStatus> {
        self.check_area(Self::POSE)?;
        if !CameraScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        if !pose.is_finite() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let set = self.poses.set(owner, frame, pose).map_err(|_| AmmlStatus::REFUSED);
        self.note_pose();
        set
    }

    /// The position written in the camera frame before `frame`. `NOT_AVAILABLE` without `camera.pose`, `REFUSED`
    /// outside the dispatch of `frame` on this thread, `FAILED` when that frame wrote no position.
    pub fn rendered_position(&self, frame: u64) -> Result<[f32; 3], AmmlStatus> {
        self.check_area(Self::POSE)?;
        if !CameraScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        let position = self.poses.rendered().ok_or(AmmlStatus::FAILED)?;
        Ok([position.x, position.y, position.z])
    }

    /// The vertical field of view in degrees and the aspect ratio of the camera of `frame`. `NOT_AVAILABLE` without
    /// `camera.projection`, `REFUSED` outside the After dispatch of `frame` on this thread, `FAILED` for a matrix
    /// without usable focal lengths.
    pub fn projection(&self, frame: u64) -> Result<(f32, f32), AmmlStatus> {
        self.check_area(Self::PROJECTION)?;
        let reader = self.projection.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (camera, after) = CALL.with(Cell::get);
        if !CameraScope::allows(frame) || !after {
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: the After dispatch runs on this thread right after the `setupCamera` call of this camera returned;
        // the game keeps the camera alive for that call.
        unsafe { reader.read(std::ptr::with_exposed_provenance(camera)) }.ok_or(AmmlStatus::FAILED)
    }

    /// The game thread rule for a game call (`camera.free`, `world.shapes`, `world.raycast`, `world.actors_in`) made
    /// from a `CameraSetup` listener: `REFUSED` when the dispatch runs on another thread than the game thread, logged
    /// once as `camera setup off the game thread`. Outside a `CameraSetup` dispatch it gives `Ok` (the call's own rules
    /// apply).
    pub fn check_game_call() -> Result<(), AmmlStatus> {
        let Some(adapter) = ADAPTER.get() else { return Ok(()) };
        if CameraScope::current().is_none() || adapter.hub().gate().on_game_thread() {
            return Ok(());
        }
        if !adapter.off_thread_logged.swap(true, Ordering::AcqRel) {
            curia::warn!("camera setup off the game thread");
        }
        Err(AmmlStatus::REFUSED)
    }

    /// The mod at `owner` is disabled: its pose is dropped, so the next write shows the game camera.
    pub fn disable_owner(&self, owner: usize) {
        self.poses.disable_owner(owner);
        self.note_pose();
    }

    /// Before `WorldLeft`: the pose and the rendered position are forgotten.
    pub(crate) fn world_left(&self) {
        self.poses.clear();
        self.note_pose();
    }
}

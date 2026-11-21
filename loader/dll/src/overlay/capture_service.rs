use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use amml_abi::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlShotLayers, AmmlStatus};
use amml_runtime::{CapturePoint, ResourceId, ResourceKind, ResourceStore, ShotBook, WorldEpoch};

use super::{CaptureJob, CaptureRead, CaptureStream, OverlayShared, PngWriter, WriterJob};
use crate::{OwnerEvent, OwnerEvents, ShutdownLatch};

/// Screenshots and capture streams: the screenshot book, the open streams, and what the render thread copies at each
/// capture point of a frame.
///
/// Capture points: `GameOnly` is copied before the first `FramePresent` listener, `WithOverlays` after the listeners
/// of the `NORMAL` band and before `READ_AFTER_WRITERS`; with no listener both points are one copy. A screenshot is
/// copied at the next present with a surface; `ScreenshotTaken` goes to the requester when the copy is read back (at
/// most `FrameCapture::MAP_WAIT_FRAMES` presents later), `ScreenshotWritten` when the worker wrote the file. A
/// screenshot whose copy fails gets both with `ok` 0, and so does a request that finds no present with a surface
/// within `SHOT_PRESENTS` presents or `SHOT_WAIT` (`expire_shots`: no device, the D3D12 path waiting for its queue, a
/// minimised window, presents that stopped) and a request still waiting when the loader shuts down (`fail_pending`).
/// While the loader shuts down, no screenshot or stream is accepted.
///
/// A stream copies every `every_nth_present` presents of the bound chain while a ring slot is free; a present with no
/// free slot is skipped and counted, never waited for. Stream ids are resources of their mod (`STREAM`); the slot
/// buffers are CPU memory, so a render device reset does not touch them.
pub struct CaptureService {
    shots: ShotBook,
    streams: Mutex<Vec<Arc<CaptureStream>>>,
    // Streams opened and not closed.
    open: AtomicUsize,
    resources: Arc<ResourceStore<usize>>,
    owner_events: Arc<OwnerEvents>,
    shutdown: Arc<ShutdownLatch>,
    writer: PngWriter,
}

impl CaptureService {
    /// The area of `screenshot`.
    pub const SCREENSHOT_AREA: &'static str = "overlay.screenshot";
    /// The area of the capture stream functions.
    pub const CAPTURE_AREA: &'static str = "overlay.capture";
    /// The resource kind of a stream id.
    pub const STREAM: ResourceKind = ResourceKind(4);
    /// The most frames one stream may hold.
    pub const MAX_RING: u8 = 8;
    /// The presents of the bound chain a screenshot request may wait for a present with a surface.
    pub const SHOT_PRESENTS: u64 = 120;
    /// How long a screenshot request may wait for a present with a surface.
    pub const SHOT_WAIT: Duration = Duration::from_secs(5);
    const NO_WORLD: WorldEpoch = WorldEpoch(0);

    pub fn new(
        resources: Arc<ResourceStore<usize>>,
        owner_events: Arc<OwnerEvents>,
        shutdown: Arc<ShutdownLatch>,
    ) -> Self {
        Self {
            shots: ShotBook::new(),
            streams: Mutex::new(Vec::new()),
            open: AtomicUsize::new(0),
            resources,
            writer: PngWriter::new(owner_events.clone()),
            owner_events,
            shutdown,
        }
    }

    /// Asks for a screenshot of the next present to `path` for the mod at `owner`; gives its id. `INVALID_ARGUMENT`
    /// for a path that is not absolute or a `layers` value that is neither `GAME_ONLY` nor `WITH_OVERLAYS`;
    /// `NOT_READY` before a swap chain is bound and while the loader shuts down.
    pub fn screenshot(&self, owner: usize, path: &Path, layers: AmmlShotLayers) -> Result<u64, AmmlStatus> {
        if !Self::known(layers) || !path.is_absolute() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let overlay = OverlayShared::global();
        if self.shutdown.is_closing() || !overlay.is_bound() {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(self.shots.request(owner, path.to_path_buf(), layers, overlay.present_count(), Instant::now()))
    }

    /// Fails the screenshot requests that found no present with a surface within `SHOT_PRESENTS` presents of the
    /// bound chain or `SHOT_WAIT`, at time `now`: each gets `ScreenshotTaken` and `ScreenshotWritten` with `ok` 0.
    /// Any thread; the render thread calls it at each present and the game thread at each frame.
    pub fn expire_shots(&self, now: Instant) {
        let present = OverlayShared::global().present_count();
        let expired = self.shots.expired(present, now, Self::SHOT_PRESENTS, Self::SHOT_WAIT);
        if expired.is_empty() {
            return;
        }
        curia::warn!("screenshot requests found no frame with a surface in time", {
            count: expired.len(),
            presents: Self::SHOT_PRESENTS,
            wait_ms: Self::SHOT_WAIT.as_millis() as u64
        });
        self.fail(expired.into_iter().map(CaptureJob::Shot).collect());
    }

    /// Fails every screenshot request that waits for its copy, for the shutdown; gives how many failed.
    pub fn fail_pending(&self) -> usize {
        let pending = self.shots.take_all();
        let count = pending.len();
        self.fail(pending.into_iter().map(CaptureJob::Shot).collect());
        count
    }

    /// Opens a capture stream for the mod at `owner`; gives its id. `INVALID_ARGUMENT` for a request smaller than v1,
    /// a `ring` outside 1 to `MAX_RING` or an unknown `layers` value; `NOT_READY` before a swap chain is bound and
    /// while the loader shuts down.
    pub fn open_capture(&self, owner: usize, request: &AmmlCaptureRequest) -> Result<u64, AmmlStatus> {
        if request.size < AmmlCaptureRequest::SIZE_V1
            || request.ring == 0
            || request.ring > Self::MAX_RING
            || !Self::known(request.layers)
        {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if self.shutdown.is_closing() || !OverlayShared::global().is_bound() {
            return Err(AmmlStatus::NOT_READY);
        }
        let id = self.resources.insert(Self::STREAM, owner, Self::NO_WORLD, 0).map_err(|_| AmmlStatus::FAILED)?.0;
        let stream = Arc::new(CaptureStream::new(id, owner, request));
        self.lock_streams().push(stream);
        self.open.fetch_add(1, Ordering::AcqRel);
        curia::info!("capture stream opened", {
            stream: id,
            ring: request.ring,
            every_nth_present: request.every_nth_present,
            layers: request.layers.0,
            max_width: request.max_width,
            max_height: request.max_height
        });
        Ok(id)
    }

    /// The oldest waiting frame of `stream`, now taken. `NOT_READY` when none waits, `STALE` for a closed or unknown
    /// stream.
    pub fn take(&self, stream: u64) -> Result<AmmlCaptureFrame, AmmlStatus> {
        let stream = self.find(stream).filter(|stream| !stream.is_closed()).ok_or(AmmlStatus::STALE)?;
        stream.take().ok_or(AmmlStatus::NOT_READY)
    }

    /// Gives back the taken frame `present` of `stream`, also after the stream was closed (its buffer is then
    /// freed). `STALE` when no such frame is taken.
    pub fn give_back(&self, stream: u64, present: u64) -> Result<(), AmmlStatus> {
        let found = self.find(stream).ok_or(AmmlStatus::STALE)?;
        let given = found.give_back(present);
        self.prune();
        if given { Ok(()) } else { Err(AmmlStatus::STALE) }
    }

    /// The slots of `stream` the render thread can fill now; 0 for a closed or unknown stream.
    pub fn free_slots(&self, stream: u64) -> u32 {
        self.find(stream).map_or(0, |stream| stream.free_slots())
    }

    /// The nth presents `stream` skipped because no slot was free; `None` for an unknown stream.
    pub fn skipped(&self, stream: u64) -> Option<u64> {
        self.find(stream).map(|stream| stream.skipped())
    }

    /// Closes `stream` and frees its frames; a taken frame is freed at its `give_back`. `STALE` for a closed or unknown
    /// stream.
    pub fn close(&self, stream: u64) -> Result<(), AmmlStatus> {
        let found = self.find(stream).ok_or(AmmlStatus::STALE)?;
        if !self.close_one(&found) {
            return Err(AmmlStatus::STALE);
        }
        self.prune();
        Ok(())
    }

    /// The mod at `owner` is released: its streams close and its waiting screenshot requests are dropped. A copy
    /// already made still reaches its file.
    pub fn end_owner(&self, owner: usize) {
        self.shots.drop_owner(owner);
        let owned: Vec<Arc<CaptureStream>> =
            self.lock_streams().iter().filter(|stream| stream.owner() == owner).cloned().collect();
        for stream in owned {
            self.close_one(&stream);
        }
        self.prune();
    }

    /// True when a frame needs a copy of the back buffer: a screenshot waits or a stream is open.
    pub fn has_work(&self) -> bool {
        !self.shots.is_empty() || self.open.load(Ordering::Acquire) > 0
    }

    /// The jobs copied at `point` of `present`: the screenshots waiting for that point, and a frame of each open
    /// stream of that point that has a free slot for this present. Render thread.
    pub fn due(&self, point: CapturePoint, present: u64) -> Vec<CaptureJob> {
        let mut jobs: Vec<CaptureJob> = self.shots.due(point).into_iter().map(CaptureJob::Shot).collect();
        if self.open.load(Ordering::Acquire) == 0 {
            return jobs;
        }
        let streams: Vec<Arc<CaptureStream>> =
            self.lock_streams().iter().filter(|stream| stream.point() == point).cloned().collect();
        for stream in streams {
            if stream.wants(present) {
                jobs.push(CaptureJob::Stream { stream: stream.id(), present });
            }
        }
        jobs
    }

    /// A copy was read back: each screenshot gets `ScreenshotTaken` and goes to the worker, each stream frame goes to
    /// the worker to fill its slot. A failed read fails its jobs.
    pub fn deliver(&self, read: CaptureRead) {
        let Some(image) = read.image else {
            self.fail(read.jobs);
            return;
        };
        for job in read.jobs {
            match job {
                CaptureJob::Shot(request) => {
                    self.owner_events.push(request.owner, OwnerEvent::ScreenshotTaken { id: request.id, ok: true });
                    self.writer.send(WriterJob::Shot { request, image: image.clone() });
                }
                CaptureJob::Stream { stream, present } => {
                    if let Some(stream) = self.find(stream) {
                        self.writer.send(WriterJob::Frame { stream, present, image: image.clone() });
                    }
                }
            }
        }
    }

    /// Jobs whose copy was not made or cannot be read: a screenshot gets `ScreenshotTaken` and `ScreenshotWritten`
    /// with `ok` 0, a stream frame gives its slot back.
    pub fn fail(&self, jobs: Vec<CaptureJob>) {
        for job in jobs {
            match job {
                CaptureJob::Shot(request) => {
                    curia::warn!("screenshot not taken", { id: request.id, path: request.path.display().to_string() });
                    self.owner_events.push(request.owner, OwnerEvent::ScreenshotTaken { id: request.id, ok: false });
                    self.owner_events.push(request.owner, OwnerEvent::ScreenshotWritten { id: request.id, ok: false });
                }
                CaptureJob::Stream { stream, present } => {
                    if let Some(stream) = self.find(stream) {
                        stream.abandon(present);
                    }
                }
            }
        }
    }

    /// Waits up to `timeout` for the worker to finish the screenshots and frames it was given; gives how many are
    /// left.
    pub fn wait_written(&self, timeout: Duration) -> usize {
        self.writer.wait_idle(timeout)
    }

    fn close_one(&self, stream: &CaptureStream) -> bool {
        if !stream.close() {
            return false;
        }
        self.open.fetch_sub(1, Ordering::AcqRel);
        let _ = self.resources.remove(ResourceId(stream.id()));
        curia::info!("capture stream closed", { stream: stream.id(), skipped: stream.skipped() });
        true
    }

    // Forgets the closed streams whose buffers are all freed.
    fn prune(&self) {
        self.lock_streams().retain(|stream| !stream.is_done());
    }

    fn find(&self, stream: u64) -> Option<Arc<CaptureStream>> {
        self.lock_streams().iter().find(|held| held.id() == stream).cloned()
    }

    fn known(layers: AmmlShotLayers) -> bool {
        layers == AmmlShotLayers::GAME_ONLY || layers == AmmlShotLayers::WITH_OVERLAYS
    }

    // The list holds plain handles; a lock poisoned by a panic is still usable.
    fn lock_streams(&self) -> MutexGuard<'_, Vec<Arc<CaptureStream>>> {
        self.streams.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

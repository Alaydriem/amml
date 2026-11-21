use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use amml_abi::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlShotLayers, AmmlStatus};

use crate::ShotLayers;

/// The screenshots and capture streams of a `FakeGame`, on the loader's rules: the same checks of a
/// request, a ring that skips a present when no slot is free, a taken frame whose pixels stay until it
/// is given back (also after a close), and `STALE` for a closed or unknown stream. Frames are kept as
/// pushed: the fake does not scale them to `max_width x max_height`. The gate and the area book are
/// checked by the table functions before a call gets here.
pub(crate) struct FakeOverlay {
    // In the high 32 bits of every screenshot and stream id, so an id of another fake is unknown here.
    tag: u32,
    next: Cell<u32>,
    presents: Cell<u64>,
    shots: RefCell<Vec<(PathBuf, ShotLayers)>>,
    streams: RefCell<Vec<FakeStream>>,
}

/// One capture stream of the fake.
struct FakeStream {
    id: u64,
    ring: u32,
    every_nth: u64,
    closed: bool,
    skipped: u64,
    filled: Vec<FakeFrame>,
    taken: Vec<FakeFrame>,
}

/// One frame of a fake stream; the pixel buffer does not move while the frame is taken.
struct FakeFrame {
    present: u64,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl FakeOverlay {
    /// The loader's largest ring.
    const MAX_RING: u8 = 8;

    pub(crate) fn new(tag: u32) -> Self {
        Self {
            tag,
            next: Cell::new(1),
            presents: Cell::new(0),
            shots: RefCell::new(Vec::new()),
            streams: RefCell::new(Vec::new()),
        }
    }

    fn next_id(&self) -> u64 {
        let id = self.next.get();
        self.next.set(id.wrapping_add(1));
        (u64::from(self.tag) << 32) | u64::from(id)
    }

    fn layers(layers: AmmlShotLayers) -> Option<ShotLayers> {
        match layers {
            AmmlShotLayers::GAME_ONLY => Some(ShotLayers::GameOnly),
            AmmlShotLayers::WITH_OVERLAYS => Some(ShotLayers::WithOverlays),
            _ => None,
        }
    }

    /// Keeps a screenshot request; `INVALID_ARGUMENT` for a path that is not absolute or unknown layers,
    /// then `NOT_READY` while no swap chain is `bound`.
    pub(crate) fn screenshot(&self, path: &str, layers: AmmlShotLayers, bound: bool) -> Result<u64, AmmlStatus> {
        let layers = Self::layers(layers).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if !Path::new(path).is_absolute() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if !bound {
            return Err(AmmlStatus::NOT_READY);
        }
        self.shots.borrow_mut().push((PathBuf::from(path), layers));
        Ok(self.next_id())
    }

    pub(crate) fn screenshots(&self) -> Vec<(PathBuf, ShotLayers)> {
        self.shots.borrow().clone()
    }

    /// Opens a stream; `INVALID_ARGUMENT` for a request shorter than v1, a ring outside 1 to 8 or unknown
    /// layers, then `NOT_READY` while no swap chain is `bound`.
    pub(crate) fn open(&self, request: &AmmlCaptureRequest, bound: bool) -> Result<u64, AmmlStatus> {
        if request.size < AmmlCaptureRequest::SIZE_V1
            || request.ring == 0
            || request.ring > Self::MAX_RING
            || Self::layers(request.layers).is_none()
        {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if !bound {
            return Err(AmmlStatus::NOT_READY);
        }
        let id = self.next_id();
        self.streams.borrow_mut().push(FakeStream {
            id,
            ring: u32::from(request.ring),
            every_nth: u64::from(request.every_nth_present.max(1)),
            closed: false,
            skipped: 0,
            filled: Vec::new(),
            taken: Vec::new(),
        });
        Ok(id)
    }

    /// One present: the present count goes up, and each open stream for which it is an nth present gets
    /// a copy of the frame in a free slot, or counts a skip.
    pub(crate) fn present(&self, width: u32, height: u32, bgra: &[u8]) {
        let present = self.presents.get() + 1;
        self.presents.set(present);
        for stream in self.streams.borrow_mut().iter_mut().filter(|stream| !stream.closed) {
            if !present.is_multiple_of(stream.every_nth) {
                continue;
            }
            if stream.free() == 0 {
                stream.skipped += 1;
                continue;
            }
            stream.filled.push(FakeFrame { present, width, height, pixels: bgra.to_vec() });
        }
    }

    pub(crate) fn present_count(&self) -> u64 {
        self.presents.get()
    }

    /// The oldest filled frame of `stream`, now taken.
    pub(crate) fn take(&self, stream: u64) -> Result<AmmlCaptureFrame, AmmlStatus> {
        let mut streams = self.streams.borrow_mut();
        let stream = streams.iter_mut().find(|held| held.id == stream && !held.closed).ok_or(AmmlStatus::STALE)?;
        if stream.filled.is_empty() {
            return Err(AmmlStatus::NOT_READY);
        }
        let frame = stream.filled.remove(0);
        let taken = AmmlCaptureFrame {
            pixels: frame.pixels.as_ptr(),
            len: frame.pixels.len(),
            width: frame.width,
            height: frame.height,
            stride: frame.width.saturating_mul(4),
            _reserved: 0,
            present_index: frame.present,
        };
        stream.taken.push(frame);
        Ok(taken)
    }

    /// Frees the taken frame `present` of `stream`, also after a close; `STALE` when it is not taken.
    pub(crate) fn give_back(&self, stream: u64, present: u64) -> Result<(), AmmlStatus> {
        let mut streams = self.streams.borrow_mut();
        let stream = streams.iter_mut().find(|held| held.id == stream).ok_or(AmmlStatus::STALE)?;
        let at = stream.taken.iter().position(|frame| frame.present == present).ok_or(AmmlStatus::STALE)?;
        stream.taken.remove(at);
        Ok(())
    }

    pub(crate) fn free_slots(&self, stream: u64) -> u32 {
        self.streams.borrow().iter().find(|held| held.id == stream && !held.closed).map_or(0, FakeStream::free)
    }

    /// Closes `stream` and frees its filled frames; a taken frame stays until its give-back.
    pub(crate) fn close(&self, stream: u64) -> Result<(), AmmlStatus> {
        let mut streams = self.streams.borrow_mut();
        let stream = streams.iter_mut().find(|held| held.id == stream && !held.closed).ok_or(AmmlStatus::STALE)?;
        stream.closed = true;
        stream.filled.clear();
        Ok(())
    }

    pub(crate) fn open_streams(&self) -> usize {
        self.streams.borrow().iter().filter(|stream| !stream.closed).count()
    }

    pub(crate) fn skipped(&self) -> u64 {
        self.streams.borrow().iter().map(|stream| stream.skipped).sum()
    }
}

impl FakeStream {
    fn free(&self) -> u32 {
        self.ring.saturating_sub((self.filled.len() + self.taken.len()) as u32)
    }
}

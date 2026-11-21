use std::sync::{Mutex, MutexGuard, PoisonError};

use amml_abi::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlShotLayers};
use amml_runtime::{CapturePoint, CaptureRing, SlotRelease};

use super::{CapturedImage, PixelConverter};

// The pixels of one ring slot: BGRA8 rows of `width * 4` bytes, and the present they show. The buffer is kept for
// the next frame of the slot and freed when the stream closes.
#[derive(Default)]
struct StreamSlot {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    present: u64,
}

/// One capture stream of a mod: its ring (`CaptureRing`) and the pixel buffer of each ring slot.
///
/// The render thread reserves a slot for an nth present (`wants`); the loader's worker converts the read copy into
/// the slot's buffer (`fill`); the mod takes the oldest filled frame and gives it back. A buffer is never touched while
/// its slot is taken, so the pointer a mod holds stays valid until its `give_back` or the stream's close. A closed
/// stream frees its free and filled buffers at once, a taken one at its `give_back` and one being written when the
/// write ends.
pub struct CaptureStream {
    id: u64,
    owner: usize,
    point: CapturePoint,
    max_width: u32,
    max_height: u32,
    ring: CaptureRing,
    slots: Mutex<Vec<StreamSlot>>,
}

impl CaptureStream {
    /// A stream `id` of the mod at `owner` for `request`, whose `ring` the caller checked.
    pub fn new(id: u64, owner: usize, request: &AmmlCaptureRequest) -> Self {
        let point = if request.layers == AmmlShotLayers::GAME_ONLY {
            CapturePoint::BeforeListeners
        } else {
            CapturePoint::AfterNormalBand
        };
        let slots = (0..request.ring).map(|_| StreamSlot::default()).collect();
        Self {
            id,
            owner,
            point,
            max_width: request.max_width,
            max_height: request.max_height,
            ring: CaptureRing::new(request.ring, request.every_nth_present),
            slots: Mutex::new(slots),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn owner(&self) -> usize {
        self.owner
    }

    /// Where in the frame the stream's copies are taken.
    pub fn point(&self) -> CapturePoint {
        self.point
    }

    /// Reserves a slot for `present` when it is an nth present and a slot is free; a skip is counted otherwise.
    pub fn wants(&self, present: u64) -> bool {
        self.ring.wants(present)
    }

    /// The copy of `present` will not be written: its slot is given back (and its buffer freed after a close).
    pub fn abandon(&self, present: u64) {
        let at = self.ring.writing(present);
        if self.ring.abandon(present) == SlotRelease::FreeNow
            && let Some(at) = at
        {
            Self::free(&mut self.lock()[at]);
        }
    }

    /// Writes `image` as the frame of `present` into its reserved slot, scaled down by the smallest whole factor that
    /// fits the stream's limit, and marks it filled. Worker thread. A copy that cannot be converted is abandoned.
    pub fn fill(&self, present: u64, image: &CapturedImage) {
        let Some(at) = self.ring.writing(present) else {
            return;
        };
        let mut pixels = std::mem::take(&mut self.lock()[at].pixels);
        let factor = PixelConverter::scale_factor(image.width, image.height, self.max_width, self.max_height);
        let Some((width, height)) = PixelConverter::to_bgra(image, factor, &mut pixels) else {
            self.lock()[at].pixels = pixels;
            self.abandon(present);
            return;
        };
        {
            let mut slots = self.lock();
            slots[at] = StreamSlot { pixels, width, height, present };
        }
        if self.ring.filled(present).is_none() {
            Self::free(&mut self.lock()[at]);
        }
    }

    /// The oldest filled frame, now taken; `None` when none waits.
    pub fn take(&self) -> Option<AmmlCaptureFrame> {
        let (at, present) = self.ring.take()?;
        let slots = self.lock();
        let slot = &slots[at];
        Some(AmmlCaptureFrame {
            pixels: slot.pixels.as_ptr(),
            len: slot.pixels.len(),
            width: slot.width,
            height: slot.height,
            stride: slot.width * PixelConverter::BYTES_PER_PIXEL as u32,
            _reserved: 0,
            present_index: present,
        })
    }

    /// Gives back the taken frame of `present`. After a close its buffer is freed. False when no frame of `present`
    /// is taken.
    pub fn give_back(&self, present: u64) -> bool {
        match self.ring.give_back(present) {
            SlotRelease::Returned => true,
            SlotRelease::FreeNow => {
                let mut slots = self.lock();
                if let Some(slot) = slots.iter_mut().find(|slot| slot.present == present) {
                    Self::free(slot);
                }
                true
            }
            SlotRelease::NotHeld => false,
        }
    }

    /// Stops the stream and frees the buffers of its free and filled slots. False when it was closed already.
    pub fn close(&self) -> bool {
        if self.ring.is_closed() {
            return false;
        }
        let freed = self.ring.close();
        let mut slots = self.lock();
        for at in freed {
            Self::free(&mut slots[at]);
        }
        true
    }

    pub fn is_closed(&self) -> bool {
        self.ring.is_closed()
    }

    /// Closed, with no slot taken or being written: every buffer is freed.
    pub fn is_done(&self) -> bool {
        self.ring.is_closed() && self.ring.free_slots() as usize == self.lock().len()
    }

    /// The slots the render thread can fill now; 0 once closed.
    pub fn free_slots(&self) -> u32 {
        if self.ring.is_closed() { 0 } else { self.ring.free_slots() }
    }

    /// The nth presents skipped because no slot was free.
    pub fn skipped(&self) -> u64 {
        self.ring.skipped()
    }

    fn free(slot: &mut StreamSlot) {
        slot.pixels = Vec::new();
        slot.width = 0;
        slot.height = 0;
    }

    // The slots hold plain buffers; a lock poisoned by a panic is still usable.
    fn lock(&self) -> MutexGuard<'_, Vec<StreamSlot>> {
        self.slots.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

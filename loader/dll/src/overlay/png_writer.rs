use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::{CapturedImage, PixelConverter, WriterJob};
use crate::{OwnerEvent, OwnerEvents};

/// The loader's capture worker: one thread, started at the first job, that encodes screenshots to PNG and writes
/// them, and converts capture stream frames into their ring slots, so the render thread never encodes or writes a
/// file. Jobs run in the order sent.
///
/// A screenshot is written as an opaque 8-bit RGB PNG marked sRGB, first to `<path>.tmp`, then renamed over `path`,
/// so the file is either the whole new picture or left as it was. The folder is not made. `ScreenshotWritten` goes to
/// the requester through `OwnerEvents`; a failure is logged as `screenshot not written` with the path and the error.
pub struct PngWriter {
    sender: Mutex<Option<Sender<WriterJob>>>,
    // The jobs sent and not finished, with the condition `wait_idle` waits on.
    pending: Arc<(Mutex<usize>, Condvar)>,
    owner_events: Arc<OwnerEvents>,
}

impl PngWriter {
    pub fn new(owner_events: Arc<OwnerEvents>) -> Self {
        Self { sender: Mutex::new(None), pending: Arc::new((Mutex::new(0), Condvar::new())), owner_events }
    }

    /// Queues `job` for the worker, which starts with the first job. When no worker can run, the job fails at once
    /// (a screenshot is reported not written, a stream frame abandoned).
    pub fn send(&self, job: WriterJob) {
        Self::count(&self.pending, 1);
        let job = {
            let mut sender = self.sender.lock().unwrap_or_else(PoisonError::into_inner);
            if sender.is_none() {
                *sender = self.spawn();
            }
            match sender.as_ref() {
                Some(live) => match live.send(job) {
                    Ok(()) => return,
                    Err(mpsc::SendError(job)) => {
                        *sender = None;
                        job
                    }
                },
                None => job,
            }
        };
        Self::fail(&self.owner_events, job, "the capture worker could not run");
        Self::count(&self.pending, -1);
    }

    /// Waits up to `timeout` for every queued job to finish and gives how many are left.
    pub fn wait_idle(&self, timeout: Duration) -> usize {
        let (count, idle) = &*self.pending;
        let deadline = Instant::now() + timeout;
        let mut left = count.lock().unwrap_or_else(PoisonError::into_inner);
        while *left > 0 {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            left = idle.wait_timeout(left, deadline - now).unwrap_or_else(PoisonError::into_inner).0;
        }
        *left
    }

    fn spawn(&self) -> Option<Sender<WriterJob>> {
        let (sender, receiver) = mpsc::channel();
        let pending = self.pending.clone();
        let owner_events = self.owner_events.clone();
        let started = std::thread::Builder::new()
            .name("amml-capture-writer".to_string())
            .spawn(move || Self::run(&receiver, &pending, &owner_events));
        match started {
            Ok(_) => Some(sender),
            Err(error) => {
                curia::error!("the capture worker could not start", { error: error.to_string() });
                None
            }
        }
    }

    fn run(receiver: &Receiver<WriterJob>, pending: &(Mutex<usize>, Condvar), owner_events: &OwnerEvents) {
        while let Ok(job) = receiver.recv() {
            let done = std::panic::catch_unwind(AssertUnwindSafe(|| Self::work(owner_events, job)));
            if done.is_err() {
                curia::error!("a capture job panicked on the worker");
            }
            Self::count(pending, -1);
        }
    }

    fn work(owner_events: &OwnerEvents, job: WriterJob) {
        match job {
            WriterJob::Shot { request, image } => {
                let written = Self::write(&request.path, &image);
                if let Err(error) = &written {
                    curia::warn!("screenshot not written", {
                        path: request.path.display().to_string(),
                        error: error.to_string()
                    });
                }
                owner_events.push(request.owner, OwnerEvent::ScreenshotWritten { id: request.id, ok: written.is_ok() });
            }
            WriterJob::Frame { stream, present, image } => stream.fill(present, &image),
        }
    }

    fn fail(owner_events: &OwnerEvents, job: WriterJob, error: &str) {
        match job {
            WriterJob::Shot { request, .. } => {
                curia::warn!("screenshot not written", { path: request.path.display().to_string(), error: error });
                owner_events.push(request.owner, OwnerEvent::ScreenshotWritten { id: request.id, ok: false });
            }
            WriterJob::Frame { stream, present, .. } => stream.abandon(present),
        }
    }

    // Writes `<path>.tmp`, flushes it to disk and renames it over `path`; on any failure the temporary file goes.
    fn write(path: &Path, image: &CapturedImage) -> io::Result<()> {
        let rgb = PixelConverter::to_rgb(image)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "the image format is not read"))?;
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        let tmp = PathBuf::from(tmp);
        let written = Self::encode(&tmp, image.width, image.height, &rgb).and_then(|()| fs::rename(&tmp, path));
        if written.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        written
    }

    fn encode(path: &Path, width: u32, height: u32, rgb: &[u8]) -> io::Result<()> {
        let mut out = BufWriter::new(File::create(path)?);
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = encoder.write_header().map_err(io::Error::other)?;
        writer.write_image_data(rgb).map_err(io::Error::other)?;
        writer.finish().map_err(io::Error::other)?;
        out.flush()?;
        out.into_inner().map_err(io::IntoInnerError::into_error)?.sync_all()
    }

    fn count(pending: &(Mutex<usize>, Condvar), change: isize) {
        let (count, idle) = pending;
        let mut count: MutexGuard<'_, usize> = count.lock().unwrap_or_else(PoisonError::into_inner);
        *count = count.saturating_add_signed(change);
        if *count == 0 {
            idle.notify_all();
        }
    }
}

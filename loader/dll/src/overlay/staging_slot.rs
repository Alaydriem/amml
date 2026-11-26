use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN};

use super::CaptureJob;

/// One staging texture of the frame capture pool and the copy it holds. The texture is kept between copies of the
/// same size and format and made again when either changes.
pub struct StagingSlot {
    pub texture: Option<ID3D11Texture2D>,
    pub width: u32,
    pub height: u32,
    pub format: DXGI_FORMAT,
    /// The jobs of the copy in the texture; empty while the slot is free.
    pub jobs: Vec<CaptureJob>,
    /// The presents the copy has waited for its non-blocking `Map`.
    pub waited: u32,
}

impl Default for StagingSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingSlot {
    pub const fn new() -> Self {
        Self { texture: None, width: 0, height: 0, format: DXGI_FORMAT_UNKNOWN, jobs: Vec::new(), waited: 0 }
    }

    /// True while a copy waits to be read.
    pub fn is_busy(&self) -> bool {
        !self.jobs.is_empty()
    }

    /// Ends the copy the slot holds, gives its jobs and keeps the texture.
    pub fn free(&mut self) -> Vec<CaptureJob> {
        self.waited = 0;
        std::mem::take(&mut self.jobs)
    }

    /// Ends the copy, gives its jobs and drops the texture.
    pub fn reset(&mut self) -> Vec<CaptureJob> {
        let jobs = self.free();
        self.texture = None;
        self.width = 0;
        self.height = 0;
        self.format = DXGI_FORMAT_UNKNOWN;
        jobs
    }
}

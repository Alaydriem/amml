use std::sync::Arc;

use windows::Win32::Foundation::{E_NOINTERFACE, E_NOTIMPL, E_OUTOFMEMORY, S_FALSE};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_CPU_ACCESS_READ, D3D11_MAP_FLAG_DO_NOT_WAIT, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_STAGING, ID3D11Device, ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::Win32::Graphics::Dxgi::DXGI_ERROR_WAS_STILL_DRAWING;
use windows::core::{HRESULT, Interface};

use super::{CaptureJob, CaptureRead, CapturedImage, PixelConverter, StagingSlot};

/// Copies the back buffer for screenshots and capture stream frames and reads the copies back with no wait on the
/// GPU. Render thread only, inside Present. A fixed pool of `SLOTS` staging textures: `request` issues one
/// `CopyResource` for all the jobs of one capture point, and `poll` tries a non-blocking `Map` of each busy texture on
/// each later Present. A mapped copy is read into a `CapturedImage` in the same call. A copy that cannot be made, or
/// mapped within `MAP_WAIT_FRAMES` Presents, gives its jobs back to fail. A multisampled back buffer, or one in a
/// format `PixelConverter` does not read, is not copied.
pub struct FrameCapture {
    slots: [StagingSlot; Self::SLOTS],
    // Each failure reason is logged once per session.
    warned: Vec<&'static str>,
}

impl Default for FrameCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameCapture {
    /// The staging textures of the pool: two capture points per frame while the GPU runs up to three frames behind,
    /// with room to spare.
    pub const SLOTS: usize = 8;
    /// The Presents a copy may wait for its `Map` before it fails.
    pub const MAP_WAIT_FRAMES: u32 = 30;

    pub const fn new() -> Self {
        Self { slots: [const { StagingSlot::new() }; Self::SLOTS], warned: Vec::new() }
    }

    /// Copies `back_buffer` into a free staging texture for `jobs`, on `context` of `device`. On D3D11On12 call it
    /// while the wrapped back buffer is acquired. `Err` gives the jobs back when no copy was made.
    pub fn request(
        &mut self,
        device: &ID3D11Device,
        context: &ID3D11DeviceContext,
        back_buffer: &ID3D11Resource,
        jobs: Vec<CaptureJob>,
    ) -> Result<(), Vec<CaptureJob>> {
        if jobs.is_empty() {
            return Ok(());
        }
        let Ok(texture) = back_buffer.cast::<ID3D11Texture2D>() else {
            self.warn("the back buffer is not a readable 2D texture", E_NOINTERFACE);
            return Err(jobs);
        };
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: `texture` is live and `desc` is writable for the call.
        unsafe { texture.GetDesc(&mut desc) };
        if desc.SampleDesc.Count > 1 {
            self.warn("the back buffer is multisampled", E_NOTIMPL);
            return Err(jobs);
        }
        if !PixelConverter::supports(desc.Format.0 as u32) {
            self.warn("the back buffer format is not read", E_NOTIMPL);
            return Err(jobs);
        }
        let Some(at) = self.slots.iter().position(|slot| !slot.is_busy()) else {
            self.warn("no free staging texture", E_OUTOFMEMORY);
            return Err(jobs);
        };
        if let Err(result) = Self::ensure_texture(device, &mut self.slots[at], &desc) {
            self.warn("the staging texture could not be made", result);
            return Err(jobs);
        }
        let slot = &mut self.slots[at];
        if let Some(staging) = &slot.texture {
            // SAFETY: both resources are live and of one device; the staging texture has the back buffer's size and
            // format, one sample, one mip and one slice.
            unsafe { context.CopyResource(staging, back_buffer) };
        }
        slot.jobs = jobs;
        slot.waited = 0;
        Ok(())
    }

    /// Tries to read each busy staging texture without waiting, and gives each copy that was read or failed.
    pub fn poll(&mut self, context: &ID3D11DeviceContext) -> Vec<CaptureRead> {
        let mut reads = Vec::new();
        for at in 0..self.slots.len() {
            if self.slots[at].is_busy()
                && let Some(read) = self.poll_one(context, at)
            {
                reads.push(read);
            }
        }
        reads
    }

    /// True while a copy waits to be read.
    pub fn in_flight(&self) -> bool {
        self.slots.iter().any(StagingSlot::is_busy)
    }

    /// Drops every staging texture, for a device loss, a surface loss or shutdown, and gives the jobs of the copies
    /// in flight, which can no longer be read.
    pub fn release(&mut self) -> Vec<CaptureJob> {
        self.slots.iter_mut().flat_map(StagingSlot::reset).collect()
    }

    // The staging texture of `slot` matches `source`, made again when the size or format changed.
    fn ensure_texture(
        device: &ID3D11Device,
        slot: &mut StagingSlot,
        source: &D3D11_TEXTURE2D_DESC,
    ) -> Result<(), HRESULT> {
        if slot.texture.is_some()
            && slot.width == source.Width
            && slot.height == source.Height
            && slot.format == source.Format
        {
            return Ok(());
        }
        slot.texture = None;
        let desc = D3D11_TEXTURE2D_DESC {
            Width: source.Width,
            Height: source.Height,
            MipLevels: 1,
            ArraySize: 1,
            Format: source.Format,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut texture: Option<ID3D11Texture2D> = None;
        // SAFETY: `desc` lives for the call and the texture has no initial data.
        unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }.map_err(|error| error.code())?;
        slot.texture = Some(texture.ok_or(E_OUTOFMEMORY)?);
        slot.width = source.Width;
        slot.height = source.Height;
        slot.format = source.Format;
        Ok(())
    }

    // `None` while the copy of slot `at` is not ready and may still wait.
    fn poll_one(&mut self, context: &ID3D11DeviceContext, at: usize) -> Option<CaptureRead> {
        let Some(texture) = self.slots[at].texture.clone() else {
            return Some(CaptureRead { jobs: self.slots[at].free(), image: None });
        };
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // SAFETY: the staging texture is live and was made with CPU read access; `mapped` is writable.
        let result =
            unsafe { context.Map(&texture, 0, D3D11_MAP_READ, D3D11_MAP_FLAG_DO_NOT_WAIT.0 as u32, Some(&mut mapped)) };
        if let Err(error) = result {
            let code = error.code();
            if code == DXGI_ERROR_WAS_STILL_DRAWING {
                self.slots[at].waited += 1;
                if self.slots[at].waited < Self::MAP_WAIT_FRAMES {
                    return None;
                }
                self.warn("the copy was not ready in time", code);
            } else {
                self.warn("the staging texture could not be mapped", code);
            }
            return Some(CaptureRead { jobs: self.slots[at].free(), image: None });
        }
        let image = Self::read_rows(&self.slots[at], &mapped);
        // SAFETY: the texture was mapped above.
        unsafe { context.Unmap(&texture, 0) };
        if image.is_none() {
            self.warn("the mapped copy is shorter than its rows", S_FALSE);
        }
        Some(CaptureRead { jobs: self.slots[at].free(), image: image.map(Arc::new) })
    }

    // The rows of the mapped copy without their padding; `None` for a null pointer or a pitch too short for a row.
    fn read_rows(slot: &StagingSlot, mapped: &D3D11_MAPPED_SUBRESOURCE) -> Option<CapturedImage> {
        if mapped.pData.is_null() || slot.width == 0 || slot.height == 0 {
            return None;
        }
        let row_bytes = slot.width as usize * PixelConverter::BYTES_PER_PIXEL;
        let pitch = mapped.RowPitch as usize;
        let height = slot.height as usize;
        if pitch < row_bytes {
            return None;
        }
        // SAFETY: a mapped texture of `height` rows holds `height` rows of `RowPitch` bytes, the last at least
        // `row_bytes` long, for as long as it is mapped.
        let source = unsafe { std::slice::from_raw_parts(mapped.pData.cast::<u8>(), (height - 1) * pitch + row_bytes) };
        let mut pixels = Vec::with_capacity(row_bytes * height);
        for row in 0..height {
            pixels.extend_from_slice(&source[row * pitch..row * pitch + row_bytes]);
        }
        Some(CapturedImage { width: slot.width, height: slot.height, format: slot.format.0 as u32, pixels })
    }

    fn warn(&mut self, reason: &'static str, result: HRESULT) {
        if self.warned.contains(&reason) {
            return;
        }
        self.warned.push(reason);
        curia::warn!("frame capture copy failed", { reason: reason, hresult: format!("{:#x}", result.0) });
    }
}

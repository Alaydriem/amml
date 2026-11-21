use std::sync::{Mutex, MutexGuard};

use amml_abi::{AmmlPoint, AmmlSize};

/// Maps window client points to back buffer points when the game renders at a size other than its client area.
pub struct BufferMap {
    // The client size and the back buffer size.
    sizes: Mutex<(AmmlSize, AmmlSize)>,
}

impl Default for BufferMap {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferMap {
    pub fn new() -> Self {
        let none = AmmlSize { width: 0, height: 0 };
        Self { sizes: Mutex::new((none, none)) }
    }

    fn lock(&self) -> MutexGuard<'_, (AmmlSize, AmmlSize)> {
        self.sizes.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn set_client(&self, size: AmmlSize) {
        self.lock().0 = size;
    }

    pub fn set_buffer(&self, size: AmmlSize) {
        self.lock().1 = size;
    }

    /// The back buffer size last set; 0 x 0 before one.
    pub fn buffer(&self) -> AmmlSize {
        self.lock().1
    }

    /// The back buffer point for a client point, scaled by buffer size over client size. `None` while either size
    /// has a zero side.
    pub fn to_buffer(&self, client: AmmlPoint) -> Option<AmmlPoint> {
        let (window, buffer) = *self.lock();
        if window.width == 0 || window.height == 0 || buffer.width == 0 || buffer.height == 0 {
            return None;
        }
        Some(AmmlPoint {
            x: client.x * buffer.width as f32 / window.width as f32,
            y: client.y * buffer.height as f32 / window.height as f32,
        })
    }
}

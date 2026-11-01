use std::ffi::c_void;

use amml_game::ActorRenderQueueLayout;

/// A flat map of fake keys and values in buffers with spare capacity, and the mirror that points at them.
pub struct FakeRenderQueue {
    keys: Vec<*mut c_void>,
    values: Vec<u8>,
    layout: ActorRenderQueueLayout,
}

impl FakeRenderQueue {
    /// The spare slots after the live elements.
    pub const SPARE: usize = 4;

    /// A queue with `keys` and `values`; the lengths may differ. Keys are fake actor addresses.
    pub fn new(keys: &[usize], values: &[u8]) -> Self {
        let mut key_buffer: Vec<*mut c_void> = keys.iter().map(|key| *key as *mut c_void).collect();
        key_buffer.resize(keys.len() + Self::SPARE, std::ptr::null_mut());
        let mut value_buffer = values.to_vec();
        value_buffer.resize(values.len() + Self::SPARE, 0);
        let layout = ActorRenderQueueLayout {
            keys_begin: key_buffer.as_mut_ptr(),
            keys_end: key_buffer.as_mut_ptr().wrapping_add(keys.len()),
            keys_cap: key_buffer.as_mut_ptr().wrapping_add(keys.len() + Self::SPARE),
            values_begin: value_buffer.as_mut_ptr(),
            values_end: value_buffer.as_mut_ptr().wrapping_add(values.len()),
            values_cap: value_buffer.as_mut_ptr().wrapping_add(values.len() + Self::SPARE),
        };
        Self { keys: key_buffer, values: value_buffer, layout }
    }

    /// A queue with null pointers, as a game vector that never held an element.
    pub fn empty() -> Self {
        let layout = ActorRenderQueueLayout {
            keys_begin: std::ptr::null_mut(),
            keys_end: std::ptr::null_mut(),
            keys_cap: std::ptr::null_mut(),
            values_begin: std::ptr::null_mut(),
            values_end: std::ptr::null_mut(),
            values_cap: std::ptr::null_mut(),
        };
        Self { keys: Vec::new(), values: Vec::new(), layout }
    }

    pub fn layout(&mut self) -> &mut ActorRenderQueueLayout {
        &mut self.layout
    }

    /// The live keys, from the begin and end pointers.
    pub fn live_keys(&self) -> Vec<usize> {
        let len = (self.layout.keys_end as usize - self.layout.keys_begin as usize) / 8;
        self.keys[..len].iter().map(|key| *key as usize).collect()
    }

    /// The live values, from the begin and end pointers.
    pub fn live_values(&self) -> Vec<u8> {
        let len = self.layout.values_end as usize - self.layout.values_begin as usize;
        self.values[..len].to_vec()
    }

    /// Whether the begin and cap pointers still point where they did at the start.
    pub fn buffers_unchanged(&self) -> bool {
        self.layout.keys_begin == self.keys.as_ptr().cast_mut()
            && self.layout.keys_cap == self.keys.as_ptr().cast_mut().wrapping_add(self.keys.len())
            && self.layout.values_begin == self.values.as_ptr().cast_mut()
            && self.layout.values_cap == self.values.as_ptr().cast_mut().wrapping_add(self.values.len())
    }
}

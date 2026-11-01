use std::mem::MaybeUninit;

/// Layout mirror of the MSVC `std::optional<T>`: the value, then the engaged flag.
#[repr(C)]
pub struct Optional<T> {
    value: MaybeUninit<T>,
    engaged: bool,
}

impl<T: Copy> Clone for Optional<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy> Copy for Optional<T> {}

impl<T: Copy> Optional<T> {
    /// An engaged optional. Only `Copy` payloads are allowed because `Optional` never drops its value.
    pub fn some(value: T) -> Optional<T> {
        Optional { value: MaybeUninit::new(value), engaged: true }
    }
}

impl<T> Optional<T> {
    pub fn none() -> Optional<T> {
        Optional { value: MaybeUninit::uninit(), engaged: false }
    }

    pub fn get(&self) -> Option<&T> {
        if !self.engaged {
            return None;
        }
        // SAFETY: `engaged` is set only when the value was initialised.
        Some(unsafe { self.value.assume_init_ref() })
    }
}

use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::game::{OptionImplLayout, RawObject};

/// An `Option*` of the game, read through its `Option::Impl`.
pub struct OptionReader<'a> {
    option: NonNull<c_void>,
    implementation: NonNull<OptionImplLayout>,
    _option: PhantomData<&'a c_void>,
}

impl<'a> OptionReader<'a> {
    /// Wraps `option`; `None` for a null option or a null `Impl` pointer.
    ///
    /// # Safety
    /// `option` must be null or a live `Option` of the game, and must stay live while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(option: *mut c_void) -> Option<Self> {
        let option = NonNull::new(option)?;
        // SAFETY: the caller guarantees a live option, whose `mImpl` pointer is at `OPTION_IMPL_OFFSET`.
        let implementation =
            unsafe { RawObject::pointer(option.as_ptr(), OptionImplLayout::OPTION_IMPL_OFFSET as u64) }?;
        // a non-null `mImpl` points to the option's `Option::Impl`; it is kept as a pointer, never as a reference,
        // because the game writes other fields of the Impl while the option lives
        let implementation = NonNull::new(implementation.cast::<OptionImplLayout>())?;
        Some(Self { option, implementation, _option: PhantomData })
    }

    /// `mID`.
    #[allow(unsafe_code)]
    pub fn id(&self) -> i32 {
        // SAFETY: `from_raw` guarantees a live option, whose Impl holds `mID` at its mirror offset.
        unsafe { (&raw const (*self.implementation.as_ptr()).id).read_unaligned() }
    }

    /// `mOptionType` (Int 5, Enum 6); `u8::MAX` when the game's value does not fit a byte.
    pub fn option_type(&self) -> u8 {
        u8::try_from(self.raw_option_type()).unwrap_or(u8::MAX)
    }

    #[allow(unsafe_code)]
    fn raw_option_type(&self) -> i32 {
        // SAFETY: `from_raw` guarantees a live option, whose Impl holds `mOptionType` at its mirror offset.
        unsafe { (&raw const (*self.implementation.as_ptr()).option_type).read_unaligned() }
    }

    /// `mSaveTag`. Bytes that are not UTF-8 are replaced.
    #[allow(unsafe_code)]
    pub fn save_tag(&self) -> String {
        // SAFETY: `from_raw` guarantees a live option; the reference covers only the 32-byte `mSaveTag`, which the
        // game writes at construction only.
        let tag = unsafe { &(*self.implementation.as_ptr()).save_tag };
        String::from_utf8_lossy(tag.as_bytes()).into_owned()
    }

    /// `IntOption::mValue`; `None` unless `mOptionType` is Int (5), because an option of another type is smaller.
    #[allow(unsafe_code)]
    pub fn int_value(&self) -> Option<i32> {
        if self.raw_option_type() != OptionImplLayout::INT_OPTION_TYPE {
            return None;
        }
        // SAFETY: `from_raw` guarantees a live option, and an Int option is an `IntOption`, which holds `mValue` at
        // this offset.
        unsafe { RawObject::read::<i32>(self.option.as_ptr(), OptionImplLayout::INT_OPTION_VALUE_OFFSET as u64) }
    }
}

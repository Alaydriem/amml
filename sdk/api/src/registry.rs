use std::ffi::c_void;

use amml_abi::{AmmlStr, ProvideFn, RequireFn};

use crate::{ApiError, Provided};

/// Tables mods provide to each other.
///
/// A provider's table must be a `static` of its DLL: the loader never unmaps a mod DLL, so the
/// table stays readable for the process. A consumer that gets `ProviderGone` for a table stops
/// using it; the provider may be disabled after that.
#[derive(Clone, Copy)]
pub struct Registry {
    provide: ProvideFn,
    require: RequireFn,
}

impl Registry {
    pub(crate) fn new(provide: ProvideFn, require: RequireFn) -> Self {
        Self { provide, require }
    }

    /// Publishes `table` as `T::NAME` at `T::MAJOR.T::MINOR`. Another mod holding the name and
    /// major gives `Refused`.
    pub fn provide<T: Provided>(&self, table: &'static T) -> Result<(), ApiError> {
        let table = std::ptr::from_ref(table).cast::<c_void>();
        // SAFETY: the name lives for the call and `table` is static.
        #[allow(unsafe_code)]
        let status = unsafe { (self.provide)(AmmlStr::from_str(T::NAME), T::MAJOR, T::MINOR, table) };
        ApiError::check(status)
    }

    /// The provider of `T::NAME` with major `T::MAJOR` and a minor of at least `T::MINOR`. `None`
    /// when there is none, or when its table is shorter than `T`.
    pub fn require<T: Provided>(&self) -> Option<&'static T> {
        let mut table: *const c_void = std::ptr::null();
        let mut minor = 0u16;
        // SAFETY: the name lives for the call and both outputs are live locals.
        #[allow(unsafe_code)]
        let status = unsafe { (self.require)(AmmlStr::from_str(T::NAME), T::MAJOR, T::MINOR, &mut table, &mut minor) };
        if !status.is_ok() || table.is_null() {
            return None;
        }
        let table = table.cast::<T>();
        // SAFETY: by the `Provided` contract the provider's table starts with a `size: u32` field;
        // the loader keeps the provider DLL mapped, so the table is readable for the process.
        #[allow(unsafe_code)]
        let size = unsafe { table.cast::<u32>().read() };
        if (size as usize) < std::mem::size_of::<T>() {
            return None;
        }
        // SAFETY: the table holds at least `size_of::<T>()` bytes laid out as `T` (the `Provided`
        // contract) and is a static of a DLL that is never unmapped.
        #[allow(unsafe_code)]
        unsafe {
            table.as_ref()
        }
    }
}

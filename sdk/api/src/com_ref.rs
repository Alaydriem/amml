use std::ffi::c_void;
use std::marker::PhantomData;

/// A borrowed COM pointer that is valid for `'f`: for a pointer of a `DrawSurface`, the
/// `FramePresent` call that gave it. It holds no reference, and it is neither `Send` nor `Sync`, so it
/// stays on the thread of that call.
///
/// `as_raw` gives the pointer. With the `d3d` feature, `with` lends it as the `windows` type for one
/// closure; the closure cannot return the reference. Do not clone (`AddRef`) the `back_buffer` or
/// `target` of a surface, and do not keep a view made from the back buffer: a reference kept past the
/// frame makes the game's next `ResizeBuffers` fail.
pub struct ComRef<'f, T> {
    // Null only for the context or target of an on-demand surface the loader could not make; `with` and the loader
    // refuse a null one.
    raw: *mut c_void,
    _borrow: PhantomData<(&'f (), *const T)>,
}

impl<'f, T> ComRef<'f, T> {
    /// A `ComRef` over `raw`; `None` for a null pointer.
    ///
    /// # Safety
    /// A non-null `raw` is a live COM object of type `T` for `'f`, used only on this thread.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn from_loader(raw: *mut c_void) -> Option<Self> {
        (!raw.is_null()).then_some(Self { raw, _borrow: PhantomData })
    }

    /// A `ComRef` that holds no object: `as_raw` gives null and `with` gives `None`.
    pub(crate) fn absent() -> Self {
        Self { raw: std::ptr::null_mut(), _borrow: PhantomData }
    }

    /// The COM pointer, valid for `'f`. Null for the `context()` or `target()` of a `DrawSurface` the
    /// loader could not make (`DrawSurface::ready()` is false): check it before passing it on.
    pub fn as_raw(&self) -> *mut c_void {
        self.raw
    }
}

#[cfg(feature = "d3d")]
impl<'f, T: windows::core::Interface> ComRef<'f, T> {
    /// A `ComRef` that borrows an interface the mod holds, for example the destination texture of
    /// `DrawSurface::copy_back_buffer`.
    pub fn from_interface(interface: &'f T) -> Self {
        Self { raw: interface.as_raw(), _borrow: PhantomData }
    }

    /// Calls `f` with the pointer as the `windows` type. `f` cannot keep the reference past the call
    /// (a `clone()` made inside it is an owned reference of the mod: never keep one of a frame's
    /// target or back buffer, or the game's resize fails). Gives `None` only for a null pointer: the
    /// context or target of a surface the loader could not make for this frame.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        // SAFETY: `raw` is null or a live `T` for `'f` (the constructors' contract), and the borrow
        // lent to `f` ends before `with` returns, inside `'f`.
        #[allow(unsafe_code)]
        let interface = unsafe { T::from_raw_borrowed(&self.raw) };
        interface.map(f)
    }
}

impl<T> Clone for ComRef<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ComRef<'_, T> {}

impl<T> std::fmt::Debug for ComRef<'_, T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("ComRef").field(&self.raw).finish()
    }
}

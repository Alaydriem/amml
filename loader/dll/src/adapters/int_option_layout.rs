/// Layout of the game's `Option`, `Option::Impl` and `IntOption` as far as the loader reads them, the same on 26.63 and
/// 70.26 (checked in the exe: the `Option::Impl` constructor stores the save tag string at 188h and the id, owner and
/// type at 1CCh, 1D0h and 1D4h; `IntOption::set` compares the value at 18h).
///
/// An `Option` holds a pointer to its `Option::Impl` at `IMPL_AT`; an `IntOption` (an int or enum option) holds its
/// value at `VALUE_AT`. These are plain reads of game memory: the caller guarantees the address.
pub struct IntOptionLayout;

impl IntOptionLayout {
    /// Offset of the `Option::Impl` pointer in an `Option`.
    pub const IMPL_AT: usize = 0x8;
    /// Offset of `mValue` in an `IntOption`.
    pub const VALUE_AT: usize = 0x18;
    /// Offset of `mSaveTag` (an MSVC `std::string`) in an `Option::Impl`.
    pub const SAVE_TAG_AT: usize = 0x188;
    /// Offset of `mID` in an `Option::Impl`.
    pub const ID_AT: usize = 0x1CC;
    /// Offset of `mOptionType` in an `Option::Impl`.
    pub const TYPE_AT: usize = 0x1D4;
    /// `mOptionType` of an int option.
    pub const INT_TYPE: i32 = 5;
    /// `mOptionType` of the `IntOption` subclass that `gfx_vsync` (option 34h) uses: its constructor runs the
    /// `IntOption` constructor on the same object first, then stores type 6, and it is the only type 6 store in the exe
    /// (survey row O5, 26.63 and 70.26).
    pub const ENUM_TYPE: i32 = 6;

    /// The `Option::Impl` of `option`; 0 when it has none.
    ///
    /// # Safety
    /// `option` is a live `Option`.
    pub unsafe fn implementation(option: usize) -> usize {
        // SAFETY: the caller guarantees the option and its Impl pointer.
        unsafe { Self::read_at::<usize>(option, Self::IMPL_AT) }
    }

    /// The id the option holds in its own Impl; `None` when it has no Impl.
    ///
    /// # Safety
    /// `option` is a live `Option`.
    pub unsafe fn id(option: usize) -> Option<i32> {
        // SAFETY: the caller guarantees the option.
        let implementation = unsafe { Self::implementation(option) };
        // SAFETY: a non-null `mImpl` points to the option's Impl.
        (implementation != 0).then(|| unsafe { Self::read_at::<i32>(implementation, Self::ID_AT) })
    }

    /// Whether `option` is an int or enum option (an `IntOption`); false when it has no Impl.
    ///
    /// # Safety
    /// `option` is a live `Option`.
    pub unsafe fn is_int(option: usize) -> bool {
        // SAFETY: the caller guarantees the option.
        let implementation = unsafe { Self::implementation(option) };
        if implementation == 0 {
            return false;
        }
        // SAFETY: a non-null `mImpl` points to the option's Impl.
        let kind = unsafe { Self::read_at::<i32>(implementation, Self::TYPE_AT) };
        kind == Self::INT_TYPE || kind == Self::ENUM_TYPE
    }

    /// The value of the `IntOption` `option`.
    ///
    /// # Safety
    /// `option` is a live `IntOption` (`is_int`).
    pub unsafe fn value(option: usize) -> i32 {
        // SAFETY: the caller guarantees an int option, which holds its value at `VALUE_AT`.
        unsafe { Self::read_at::<i32>(option, Self::VALUE_AT) }
    }

    /// Reads a `T` at `base + offset` of game memory.
    ///
    /// # Safety
    /// `base + offset` is readable game memory that holds a `T`.
    pub(crate) unsafe fn read_at<T: Copy>(base: usize, offset: usize) -> T {
        // SAFETY: the caller guarantees the address.
        unsafe { std::ptr::with_exposed_provenance::<T>(base.wrapping_add(offset)).read_unaligned() }
    }
}

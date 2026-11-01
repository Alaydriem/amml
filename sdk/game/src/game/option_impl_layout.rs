use std::mem::offset_of;

use amml_msvc::MsvcString;

/// Layout mirror of the game's `Option::Impl` (0x2A0 bytes), the same on 26.51 and 26.60. Only the fields the mods read
/// are named. `Option::mImpl` is a pointer at `OPTION_IMPL_OFFSET` in the option, and `IntOption::mValue` is at
/// `INT_OPTION_VALUE_OFFSET` in the option; both are outside this struct.
#[repr(C)]
pub struct OptionImplLayout {
    _head: [u8; 0x188],       // 0x000
    pub save_tag: MsvcString, // 0x188 mSaveTag
    _telemetry: [u8; 0x24],   // 0x1A8 mTelemetryProperty, mTelemetryTag
    pub id: i32,              // 0x1CC mID
    _owner_type: i32,         // 0x1D0 mOwnerType
    pub option_type: i32,     // 0x1D4 mOptionType (Int 5, Enum 6)
    _tail: [u8; 0xC8],        // 0x1D8
}

impl OptionImplLayout {
    /// Offset of the `Option::Impl` pointer in an `Option`.
    pub const OPTION_IMPL_OFFSET: usize = 0x8;
    /// `mOptionType` of an `IntOption`.
    pub const INT_OPTION_TYPE: i32 = 5;
    /// Offset of `mValue` in an `IntOption`.
    pub const INT_OPTION_VALUE_OFFSET: usize = 0x18;
}

const _: () = assert!(offset_of!(OptionImplLayout, save_tag) == 0x188);
const _: () = assert!(offset_of!(OptionImplLayout, id) == 0x1CC);
const _: () = assert!(offset_of!(OptionImplLayout, option_type) == 0x1D4);
const _: () = assert!(size_of::<OptionImplLayout>() == 0x2A0);

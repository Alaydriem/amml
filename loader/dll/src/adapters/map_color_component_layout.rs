use std::mem::offset_of;

use crate::MceColor;

/// The game's `BlockMapColorComponent`, as far as the map colour reads it: the colour and the tint method, the index of
/// the biome policy that tints the colour. Read from the exe on both builds: the 26.63 map colour function (0x63fbe30)
/// passes the byte at 10h to `getMapPolicy` and multiplies the four floats at 0.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct MapColorComponentLayout {
    pub base: MceColor, // 0x00 mMapColor
    pub tint: u8,       // 0x10 mTintMethod
}

const _: () = assert!(offset_of!(MapColorComponentLayout, tint) == 0x10);

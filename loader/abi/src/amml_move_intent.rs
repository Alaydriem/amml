/// The movement a mod asks for: axes in -1.0 to 1.0 and the held actions as 0 or 1.
///
/// A fixed layout with no `size` field: `set_move` reads exactly 12 bytes. A new field needs a new entry
/// point in the input table, never a change to this struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmlMoveIntent {
    pub forward: f32,
    pub left: f32,
    pub jump: u8,
    pub sneak: u8,
    pub sprint: u8,
    pub _reserved: u8,
}

const _: () = assert!(core::mem::size_of::<AmmlMoveIntent>() == 12);

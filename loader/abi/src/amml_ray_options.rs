/// How `raycast` walks: `every_block` 1 makes every block a candidate (a flower, tall grass, a torch), 0 only the blocks
/// whose material is solid; `outline` 1 clips each candidate against its selection outline, 0 against its collision
/// shape; `max_steps` is the most blocks the walk visits, 0 for the game's own count (200).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlRayOptions {
    pub every_block: u8,
    pub outline: u8,
    pub _reserved: [u8; 2],
    pub max_steps: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlRayOptions>() == 8);

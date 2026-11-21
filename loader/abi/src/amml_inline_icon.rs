/// An inline chat icon of the loader. `id` is never given twice in a process, `generation` counts the times the loader
/// freed every icon (a world leave): an icon of an older generation is `STALE`. `glyph` is the private-use code point
/// (`U+E110..=U+E1FF`) that draws the icon in the default font.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlInlineIcon {
    pub id: u64,
    pub generation: u32,
    pub glyph: u32,
}

const _: () = assert!(core::mem::size_of::<AmmlInlineIcon>() == 16);

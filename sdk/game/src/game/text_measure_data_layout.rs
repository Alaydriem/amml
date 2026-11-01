use std::mem::offset_of;

/// Layout mirror of the game's `TextMeasureData` (16 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct TextMeasureDataLayout {
    pub font_size: f32,          // 0x00 fontSize
    pub line_padding: f32,       // 0x04 linePadding
    pub render_shadow: bool,     // 0x08 renderShadow
    pub show_color_symbol: bool, // 0x09 showColorSymbol
    pub hide_hyphen: bool,       // 0x0A hideHyphen
    _padding: u8,                // 0x0B
    pub alignment: u32,          // 0x0C alignment
}

impl TextMeasureDataLayout {
    /// A measure for one line at `font_size`: no line padding, no color symbols, no hyphen hiding, alignment 0.
    pub fn new(font_size: f32, render_shadow: bool) -> Self {
        Self {
            font_size,
            line_padding: 0.0,
            render_shadow,
            show_color_symbol: false,
            hide_hyphen: false,
            _padding: 0,
            alignment: 0,
        }
    }
}

const _: () = assert!(offset_of!(TextMeasureDataLayout, render_shadow) == 0x08);
const _: () = assert!(offset_of!(TextMeasureDataLayout, alignment) == 0x0C);
const _: () = assert!(size_of::<TextMeasureDataLayout>() == 16);

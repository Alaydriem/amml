/// CPU copy of the 256 x 256 RGBA8 page of the private-use font glyphs `U+E100..U+E1FF`. Row 0 (16 pixel rows, the
/// code points `U+E100..U+E10F`) holds the vanilla glyphs; rows 1 to 15 hold one 16 x 16 cell for each of the 240 icon
/// code points `U+E110..U+E1FF`, drawn from the top left of the cell.
///
/// The page is marked dirty on every change, so the upload side sends it once for each change.
pub struct GlyphPage {
    pixels: Vec<u8>,
    dirty: bool,
}

impl GlyphPage {
    /// The width and the height of the page in pixels.
    pub const SIDE: u32 = 256;
    /// The width and the height of one cell in pixels.
    pub const CELL_SIDE: u32 = 16;
    /// The first code point of the page; its row 0 is the vanilla row.
    pub const FIRST_CODE_POINT: u32 = 0xE100;
    /// The first code point that is an icon cell.
    pub const FIRST_CELL: u32 = Self::FIRST_CODE_POINT + 16;
    /// The last code point of the page.
    pub const LAST_CODE_POINT: u32 = Self::FIRST_CODE_POINT + 255;
    /// The size of row 0 in bytes.
    pub const ROW0_BYTES: usize = (Self::SIDE * Self::CELL_SIDE * 4) as usize;

    /// A page with `row0` in its first 16 pixel rows and blank cells. A `row0` of another size is left out.
    pub fn new(row0: &[u8]) -> Self {
        let mut pixels = vec![0u8; (Self::SIDE * Self::SIDE * 4) as usize];
        if row0.len() == Self::ROW0_BYTES {
            pixels[..Self::ROW0_BYTES].copy_from_slice(row0);
        }
        Self { pixels, dirty: true }
    }

    // The pixel position of the top left corner of the cell of `code_point`; `None` outside the icon cells.
    fn origin(code_point: u32) -> Option<(usize, usize)> {
        if !(Self::FIRST_CELL..=Self::LAST_CODE_POINT).contains(&code_point) {
            return None;
        }
        let cell = code_point - Self::FIRST_CODE_POINT;
        Some(((cell % 16 * Self::CELL_SIDE) as usize, (cell / 16 * Self::CELL_SIDE) as usize))
    }

    /// Clears the cell of `code_point`, then copies the `width` x `height` RGBA image into its top left corner. False,
    /// and no change, for a code point that is not an icon cell, an image over 16 x 16, or pixels that do not fit.
    pub fn write_cell(&mut self, code_point: u32, width: u32, height: u32, rgba: &[u8]) -> bool {
        let Some((left, top)) = Self::origin(code_point) else { return false };
        let fits = width <= Self::CELL_SIDE && height <= Self::CELL_SIDE;
        if !fits || u64::try_from(rgba.len()) != Ok(u64::from(width) * u64::from(height) * 4) {
            return false;
        }
        self.blank(left, top);
        let row_bytes = width as usize * 4;
        for row in 0..height as usize {
            let from = row * row_bytes;
            let to = ((top + row) * Self::SIDE as usize + left) * 4;
            self.pixels[to..to + row_bytes].copy_from_slice(&rgba[from..from + row_bytes]);
        }
        self.dirty = true;
        true
    }

    /// Blanks the cell of `code_point`. False for a code point that is not an icon cell.
    pub fn clear_cell(&mut self, code_point: u32) -> bool {
        let Some((left, top)) = Self::origin(code_point) else { return false };
        self.blank(left, top);
        self.dirty = true;
        true
    }

    fn blank(&mut self, left: usize, top: usize) {
        let side = Self::CELL_SIDE as usize;
        for row in 0..side {
            let at = ((top + row) * Self::SIDE as usize + left) * 4;
            self.pixels[at..at + side * 4].fill(0);
        }
    }

    /// Marks the page changed again, after an upload that did not happen.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// The pixels when the page changed since the last call, and clears the mark.
    pub fn take_dirty(&mut self) -> Option<&[u8]> {
        if self.dirty {
            self.dirty = false;
            Some(&self.pixels)
        } else {
            None
        }
    }
}

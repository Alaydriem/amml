use amml_runtime::GlyphPage;

fn row0() -> Vec<u8> {
    (0..GlyphPage::ROW0_BYTES).map(|index| (index % 251) as u8 | 1).collect()
}

fn pixel(page: &[u8], x: usize, y: usize) -> [u8; 4] {
    let at = (y * GlyphPage::SIDE as usize + x) * 4;
    [page[at], page[at + 1], page[at + 2], page[at + 3]]
}

#[test]
fn row_0_is_kept() {
    let row0 = row0();
    let mut page = GlyphPage::new(&row0);

    let pixels = page.take_dirty().unwrap();

    assert_eq!(pixels.len(), (GlyphPage::SIDE * GlyphPage::SIDE * 4) as usize);
    assert_eq!(&pixels[..GlyphPage::ROW0_BYTES], row0.as_slice());
    assert!(pixels[GlyphPage::ROW0_BYTES..].iter().all(|byte| *byte == 0));
}

#[test]
fn a_row_0_of_the_wrong_size_is_left_out() {
    let mut page = GlyphPage::new(&[1, 2, 3]);

    assert!(page.take_dirty().unwrap().iter().all(|byte| *byte == 0));
}

#[test]
fn cell_write_lands_at_its_grid_place() {
    let mut page = GlyphPage::new(&row0());
    page.take_dirty();

    // U+E111 is cell 17: row 1, column 1.
    assert!(page.write_cell(0xE111, 2, 2, &[9; 16]));

    let pixels = page.take_dirty().unwrap().to_vec();
    assert_eq!(pixel(&pixels, 16, 16), [9; 4]);
    assert_eq!(pixel(&pixels, 17, 17), [9; 4]);
    assert_eq!(pixel(&pixels, 18, 16), [0; 4]);
    assert_eq!(pixel(&pixels, 15, 16), [0; 4]);
    assert_eq!(pixel(&pixels, 16, 15), pixel(&row0(), 16, 15));
}

#[test]
fn a_new_image_clears_the_old_pixels_of_its_cell() {
    let mut page = GlyphPage::new(&row0());
    assert!(page.write_cell(0xE110, 16, 16, &[7; 1024]));
    assert!(page.write_cell(0xE110, 1, 1, &[5; 4]));

    let pixels = page.take_dirty().unwrap().to_vec();

    assert_eq!(pixel(&pixels, 0, 16), [5; 4]);
    assert_eq!(pixel(&pixels, 1, 16), [0; 4]);
    assert_eq!(pixel(&pixels, 15, 31), [0; 4]);
}

#[test]
fn clearing_a_cell_blanks_it() {
    let mut page = GlyphPage::new(&row0());
    assert!(page.write_cell(0xE1FF, 16, 16, &[7; 1024]));
    assert!(page.clear_cell(0xE1FF));

    let pixels = page.take_dirty().unwrap().to_vec();

    assert_eq!(pixel(&pixels, 255, 255), [0; 4]);
    assert_eq!(pixel(&pixels, 240, 240), [0; 4]);
}

#[test]
fn vanilla_row_and_other_planes_are_not_cells() {
    let mut page = GlyphPage::new(&row0());
    page.take_dirty();

    assert!(!page.write_cell(0xE10F, 1, 1, &[1; 4]));
    assert!(!page.write_cell(0xE200, 1, 1, &[1; 4]));
    assert!(!page.write_cell(0xE110, 17, 1, &[1; 68]));
    assert!(!page.write_cell(0xE110, 2, 2, &[1; 4]));
    assert!(!page.clear_cell(0xE000));
    assert!(page.take_dirty().is_none());
}

#[test]
fn a_page_taken_once_is_clean_until_the_next_change() {
    let mut page = GlyphPage::new(&row0());
    assert!(page.take_dirty().is_some());
    assert!(page.take_dirty().is_none());

    page.mark_dirty();

    assert!(page.take_dirty().is_some());
}

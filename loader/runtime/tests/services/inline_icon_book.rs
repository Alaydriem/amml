use amml_runtime::{IconRefusal, InlineIconBook};

const OWNER: usize = 1;
const OTHER: usize = 2;

fn book() -> InlineIconBook {
    InlineIconBook::new(&vec![0; 256 * 16 * 4])
}

fn image(side: u32) -> Vec<u8> {
    vec![200; (side * side * 4) as usize]
}

fn small(book: &InlineIconBook) -> Result<amml_runtime::IconKey, IconRefusal> {
    book.register(OWNER, 8, 8, &image(8), 9.0)
}

#[test]
fn first_icon_is_e110() {
    let book = book();

    let icon = small(&book).unwrap();

    assert_eq!(icon.code_point, 0xE110);
    assert_eq!(book.advance_of(0xE110), Some(9.0));
    assert!(book.has_icons());
}

#[test]
fn released_code_point_is_reused_only_when_full() {
    let book = book();
    let first = small(&book).unwrap();
    let second = small(&book).unwrap();
    book.release(OWNER, first).unwrap();
    assert_eq!(second.code_point, 0xE111);

    // A cell that was never used comes before the released one.
    let third = small(&book).unwrap();
    assert_eq!(third.code_point, 0xE112);
    assert_eq!(book.advance_of(0xE110), None);

    // Use every cell that was never used; the released cell is then the only free one.
    for _ in 0..(InlineIconBook::CELLS - 3) {
        small(&book).unwrap();
    }
    assert_eq!(book.len(), InlineIconBook::CELLS - 1);
    assert_eq!(small(&book).unwrap().code_point, 0xE110);
    assert_eq!(small(&book), Err(IconRefusal::Refused));
}

#[test]
fn the_oldest_released_cell_is_reused_first() {
    let book = book();
    let icons: Vec<_> = (0..InlineIconBook::CELLS).map(|_| small(&book).unwrap()).collect();
    book.release(OWNER, icons[5]).unwrap();
    book.release(OWNER, icons[2]).unwrap();

    assert_eq!(small(&book).unwrap().code_point, icons[5].code_point);
    assert_eq!(small(&book).unwrap().code_point, icons[2].code_point);
}

#[test]
fn icon_241_is_refused() {
    let book = book();
    for _ in 0..InlineIconBook::CELLS {
        small(&book).unwrap();
    }

    assert_eq!(small(&book), Err(IconRefusal::Refused));
    assert_eq!(book.len(), InlineIconBook::CELLS);
}

#[test]
fn advance_above_16_is_refused() {
    let book = book();

    assert_eq!(book.register(OWNER, 8, 8, &image(8), 16.5), Err(IconRefusal::Refused));
    assert_eq!(book.register(OWNER, 8, 8, &image(8), 0.0), Err(IconRefusal::Refused));
    assert_eq!(book.register(OWNER, 8, 8, &image(8), -1.0), Err(IconRefusal::Refused));
    assert_eq!(book.register(OWNER, 8, 8, &image(8), f32::NAN), Err(IconRefusal::Refused));
    assert!(book.register(OWNER, 8, 8, &image(8), 16.0).is_ok());
}

#[test]
fn an_image_over_16_by_16_is_refused_and_a_broken_one_is_bad() {
    let book = book();

    assert_eq!(book.register(OWNER, 17, 16, &vec![0; 17 * 16 * 4], 9.0), Err(IconRefusal::Refused));
    assert_eq!(book.register(OWNER, 0, 4, &[], 9.0), Err(IconRefusal::BadImage));
    assert_eq!(book.register(OWNER, 4, 4, &[0; 12], 9.0), Err(IconRefusal::BadImage));
    assert!(!book.has_icons());
}

#[test]
fn register_writes_the_page_and_release_blanks_it() {
    let book = book();
    let page = book.take_page().unwrap();
    assert!(book.take_page().is_none());
    assert_eq!(page.len(), 256 * 256 * 4);

    let icon = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();
    let page = book.take_page().unwrap();
    assert_eq!(&page[(16 * 256) * 4..(16 * 256) * 4 + 4], &[255; 4]);

    book.release(OWNER, icon).unwrap();
    let page = book.take_page().unwrap();
    assert_eq!(&page[(16 * 256) * 4..(16 * 256) * 4 + 4], &[0; 4]);
    assert!(!book.has_icons());
    assert_eq!(book.advance_of(icon.code_point), None);
}

#[test]
fn an_update_changes_the_image_and_keeps_the_advance() {
    let book = book();
    let icon = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();
    book.take_page();

    book.update(OWNER, icon, 1, 1, &[7; 4]).unwrap();

    let page = book.take_page().unwrap();
    assert_eq!(&page[(16 * 256) * 4..(16 * 256) * 4 + 4], &[7; 4]);
    assert_eq!(&page[(16 * 256) * 4 + 4..(16 * 256) * 4 + 8], &[0; 4]);
    assert_eq!(book.advance_of(icon.code_point), Some(3.0));
    assert_eq!(book.update(OWNER, icon, 17, 1, &[7; 68]), Err(IconRefusal::Refused));
    assert_eq!(book.update(OWNER, icon, 1, 1, &[7; 3]), Err(IconRefusal::BadImage));
}

#[test]
fn another_owner_cannot_change_an_icon() {
    let book = book();
    let icon = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();

    assert_eq!(book.update(OTHER, icon, 1, 1, &[7; 4]), Err(IconRefusal::Refused));
    assert_eq!(book.release(OTHER, icon), Err(IconRefusal::Refused));
    assert_eq!(book.len(), 1);
}

#[test]
fn a_released_icon_is_stale() {
    let book = book();
    let icon = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();
    book.release(OWNER, icon).unwrap();

    assert_eq!(book.release(OWNER, icon), Err(IconRefusal::Stale));
    assert_eq!(book.update(OWNER, icon, 1, 1, &[7; 4]), Err(IconRefusal::Stale));
}

#[test]
fn a_world_leave_frees_every_icon_and_makes_the_keys_stale() {
    let book = book();
    let icon = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();
    book.take_page();

    assert_eq!(book.world_left(), 1);

    assert!(!book.has_icons());
    assert_eq!(book.advance_of(icon.code_point), None);
    assert_eq!(book.update(OWNER, icon, 1, 1, &[7; 4]), Err(IconRefusal::Stale));
    assert_eq!(book.release(OWNER, icon), Err(IconRefusal::Stale));
    assert!(book.take_page().is_some());
    // The new generation hands out the next unused code point.
    assert_eq!(book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap().code_point, 0xE111);
}

#[test]
fn disabling_an_owner_frees_only_its_icons() {
    let book = book();
    let mine = book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();
    let other = book.register(OTHER, 2, 2, &[255; 16], 3.0).unwrap();

    assert_eq!(book.disable_owner(OWNER), 1);

    assert_eq!(book.advance_of(mine.code_point), None);
    assert_eq!(book.advance_of(other.code_point), Some(3.0));
}

#[test]
fn other_code_points_have_no_advance() {
    let book = book();
    book.register(OWNER, 2, 2, &[255; 16], 3.0).unwrap();

    assert_eq!(book.advance_of(0xE10F), None);
    assert_eq!(book.advance_of(0xE200), None);
    assert_eq!(book.advance_of(u32::MAX), None);
    assert_eq!(book.advance_of(0x41), None);
}

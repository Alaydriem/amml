use amml_runtime::BlockTypeBook;

#[test]
fn same_type_address_gives_the_same_id() {
    let book = BlockTypeBook::new();

    let first = book.id_of(0x1000, 1, || Some(b"minecraft:stone".to_vec()));
    let again = book.id_of(0x1000, 1, || panic!("the name is read once"));

    assert_eq!(first, again);
    assert_eq!(book.name(first).as_deref(), Some("minecraft:stone"));
}

#[test]
fn ids_are_never_reused() {
    let book = BlockTypeBook::new();

    let ids: Vec<u32> = (0..1000).map(|n| book.id_of(0x1000 + n * 0x100, 1, || None)).collect();

    assert_eq!(ids.first(), Some(&1));
    assert!(ids.windows(2).all(|pair| pair[1] == pair[0] + 1));
    assert_eq!(book.id_of(0x1000 + 64 * 0x100, 1, || None), ids[64]);
    assert_eq!(book.id_of(0x1000 + 1000 * 0x100, 1, || None), 1001);
}

#[test]
fn non_utf8_name_reads_none() {
    let book = BlockTypeBook::new();

    let id = book.id_of(0x1000, 1, || Some(vec![0x6d, 0xff, 0xfe]));

    assert_ne!(id, BlockTypeBook::NONE);
    assert_eq!(book.name(id), None);
}

#[test]
fn a_null_address_is_no_block_type() {
    let book = BlockTypeBook::new();

    assert_eq!(book.id_of(0, 1, || Some(b"x".to_vec())), BlockTypeBook::NONE);
    assert_eq!(book.name(BlockTypeBook::NONE), None);
}

#[test]
fn same_address_in_a_new_epoch_with_the_same_name_keeps_the_id() {
    let book = BlockTypeBook::new();
    let first = book.id_of(0x1000, 1, || Some(b"custom:ore".to_vec()));

    let again = book.id_of(0x1000, 2, || Some(b"custom:ore".to_vec()));

    assert_eq!(again, first);
    assert_eq!(book.name(first).as_deref(), Some("custom:ore"));
}

#[test]
fn same_address_in_a_new_epoch_with_another_name_gives_a_new_id() {
    let book = BlockTypeBook::new();
    let first = book.id_of(0x1000, 1, || Some(b"custom:ore".to_vec()));

    let second = book.id_of(0x1000, 2, || Some(b"custom:glass".to_vec()));
    let third = book.id_of(0x2000, 2, || Some(b"custom:ore".to_vec()));

    assert_ne!(second, first);
    assert_ne!(third, first);
    assert_eq!(book.name(first).as_deref(), Some("custom:ore"));
    assert_eq!(book.name(second).as_deref(), Some("custom:glass"));
    assert_eq!(book.id_of(0x1000, 2, || panic!("checked in this epoch")), second);
}

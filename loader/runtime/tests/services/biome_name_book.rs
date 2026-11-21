use amml_runtime::BiomeNameBook;

#[test]
fn name_is_read_once_per_id() {
    let book = BiomeNameBook::new();

    book.meet(4, 1, || Some(b"plains".to_vec()));
    book.meet(4, 1, || panic!("the name is read once"));

    assert_eq!(book.name(4, 1).as_deref(), Some("plains"));
}

#[test]
fn unknown_id_reads_none() {
    let book = BiomeNameBook::new();
    book.meet(4, 1, || Some(b"plains".to_vec()));

    assert_eq!(book.name(5, 1), None);
}

#[test]
fn unreadable_name_reads_none() {
    let book = BiomeNameBook::new();

    book.meet(1, 1, || Some(vec![0x6d, 0xff]));
    book.meet(2, 1, || Some(Vec::new()));
    book.meet(3, 1, || None);

    assert_eq!((book.name(1, 1), book.name(2, 1), book.name(3, 1)), (None, None, None));
}

#[test]
fn a_new_world_epoch_reads_the_name_again() {
    let book = BiomeNameBook::new();
    book.meet(70, 1, || Some(b"custom:swamp".to_vec()));

    book.meet(70, 2, || Some(b"custom:dunes".to_vec()));

    assert_eq!(book.name(70, 2).as_deref(), Some("custom:dunes"));
}

#[test]
fn an_id_not_met_in_the_current_epoch_reads_none() {
    let book = BiomeNameBook::new();
    book.meet(70, 1, || Some(b"custom:swamp".to_vec()));

    assert_eq!(book.name(70, 2), None);
}

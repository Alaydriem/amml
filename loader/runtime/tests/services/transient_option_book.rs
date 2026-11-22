use amml_runtime::{OptionRestore, TransientOptionBook, TransientOptionRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;
const VSYNC: u32 = 1;
const LIMIT: u32 = 2;
const OPTION: i32 = 0xF6;

#[test]
fn release_restores_the_value_from_before_the_first_set() {
    let mut book = TransientOptionBook::new();
    // The option holds 1; the mod sets 0, then 5 (the option held 0 at the second set).
    book.set(OWNER, VSYNC, OPTION, 1, 0).unwrap();
    book.set(OWNER, VSYNC, OPTION, 0, 5).unwrap();

    assert_eq!(book.holder(VSYNC), Some(OWNER));
    assert_eq!(book.release(OWNER, VSYNC, 5), Ok(OptionRestore::Restore(1)));
    assert_eq!(book.holder(VSYNC), None);
}

#[test]
fn release_keeps_a_value_the_user_set() {
    let mut book = TransientOptionBook::new();
    book.set(OWNER, VSYNC, OPTION, 1, 0).unwrap();

    // The user changed the option to 2 while the mod's value was set: nothing is written back.
    assert_eq!(book.release(OWNER, VSYNC, 2), Ok(OptionRestore::Keep));
    assert_eq!(book.holder(VSYNC), None);
}

#[test]
fn second_owner_is_refused() {
    let mut book = TransientOptionBook::new();
    book.set(OWNER, VSYNC, OPTION, 1, 0).unwrap();

    assert_eq!(book.set(OTHER, VSYNC, OPTION, 0, 3), Err(TransientOptionRefusal::Taken));
    assert_eq!(book.release(OTHER, VSYNC, 0), Err(TransientOptionRefusal::Taken));
    // Another key is free, and a release of a key nobody holds writes nothing.
    assert_eq!(book.set(OTHER, LIMIT, OPTION, 60, 0), Ok(()));
    assert_eq!(book.release(OWNER, 9, 0), Ok(OptionRestore::Nothing));
}

#[test]
fn forgotten_holds_free_the_keys_with_no_restore() {
    let mut book = TransientOptionBook::new();
    book.set(OWNER, VSYNC, OPTION, 1, 0).unwrap();
    book.set(OTHER, LIMIT, OPTION, 60, 0).unwrap();

    book.forget_owner(OWNER);

    assert_eq!(book.holder(VSYNC), None);
    assert_eq!(book.holder(LIMIT), Some(OTHER));
    assert_eq!(book.set(OTHER, VSYNC, OPTION, 1, 2), Ok(()));
}

#[test]
fn the_keys_of_an_owner_are_listed_for_its_disable() {
    let mut book = TransientOptionBook::new();
    book.set(OWNER, VSYNC, OPTION, 1, 0).unwrap();
    book.set(OWNER, LIMIT, OPTION, 60, 0).unwrap();
    book.set(OTHER, 7, OPTION, 1, 2).unwrap();

    assert_eq!(book.keys_of(OWNER), vec![VSYNC, LIMIT]);
    assert_eq!(book.keys_of(OTHER), vec![7]);
}

#[test]
fn the_hold_keeps_the_option_of_the_first_set() {
    let mut book = TransientOptionBook::new();
    book.set(OWNER, LIMIT, 0xF6, 60, 0).unwrap();
    // A second set while another game option is effective keeps the first option.
    book.set(OWNER, LIMIT, 0xF7, 30, 5).unwrap();

    assert_eq!(book.held_option(LIMIT), Some(0xF6));
    assert_eq!(book.release(OWNER, LIMIT, 5), Ok(OptionRestore::Restore(60)));
    assert_eq!(book.held_option(LIMIT), None);
}

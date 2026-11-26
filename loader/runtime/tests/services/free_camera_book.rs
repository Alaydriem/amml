use amml_runtime::{FreeCameraBook, FreeCameraRefusal, FreeCameraRelease};

const OWNER: usize = 1;
const OTHER: usize = 2;

#[test]
fn second_owner_is_refused() {
    let mut book = FreeCameraBook::new();
    book.set(OWNER).unwrap();

    assert_eq!(book.set(OTHER), Err(FreeCameraRefusal::Taken));
    assert_eq!(book.release(OTHER), Err(FreeCameraRefusal::Taken));

    // The owner may move the camera again and release it.
    assert_eq!(book.set(OWNER), Ok(()));
    assert_eq!(book.release(OWNER), Ok(true));
    assert_eq!(book.owner(), None);
    assert_eq!(book.set(OTHER), Ok(()));
}

#[test]
fn release_without_an_owner_sends_nothing() {
    let mut book = FreeCameraBook::new();
    assert_eq!(book.release(OWNER), Ok(false));
}

#[test]
fn disable_releases_the_free_camera() {
    let mut book = FreeCameraBook::new();
    book.set(OWNER).unwrap();

    // Another mod's disable changes nothing.
    assert_eq!(book.disable(OTHER, true), FreeCameraRelease { clear: false, drop_packet: false });
    assert_eq!(book.owner(), Some(OWNER));

    // The owner's disable while the game runs on the game thread sends the clear and gives the packet back.
    assert_eq!(book.disable(OWNER, true), FreeCameraRelease { clear: true, drop_packet: true });
    assert_eq!(book.owner(), None);
    assert_eq!(book.set(OTHER), Ok(()));

    // Off the game thread, or once the game stopped, nothing is sent; the packet still goes back.
    assert_eq!(book.disable(OTHER, false), FreeCameraRelease { clear: false, drop_packet: true });
    assert_eq!(book.owner(), None);
}

#[test]
fn world_leave_forgets_the_owner() {
    let mut book = FreeCameraBook::new();
    book.set(OWNER).unwrap();

    book.world_left();

    assert_eq!(book.owner(), None);
    assert_eq!(book.set(OTHER), Ok(()));
    // The old owner's disable in the new world sends nothing.
    assert_eq!(book.disable(OWNER, true), FreeCameraRelease { clear: false, drop_packet: false });
}

#[test]
fn a_server_instruction_forgets_the_owner() {
    let mut book = FreeCameraBook::new();
    book.set(OWNER).unwrap();

    book.server_instruction();

    // The server took the camera: another mod may take it, and the old owner's disable sends no clear.
    assert_eq!(book.owner(), None);
    assert_eq!(book.disable(OWNER, true), FreeCameraRelease { clear: false, drop_packet: false });
    assert_eq!(book.set(OTHER), Ok(()));
}

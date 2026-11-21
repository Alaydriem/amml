use amml_runtime::ScreenBook;

const CONTROLLER: usize = 0x1000;
const VIEW: usize = 0x2000;

#[test]
fn first_event_gives_a_new_id() {
    let book = ScreenBook::new();

    let (screen, view) = book.note(CONTROLLER, VIEW);

    assert_ne!(screen, 0);
    assert_ne!(view, 0);
    assert_ne!(screen, view);
    assert_eq!(book.note(CONTROLLER, VIEW), (screen, view));
    assert_eq!(book.view_address(view), Some(VIEW));
}

#[test]
fn retired_id_never_comes_back() {
    let book = ScreenBook::new();
    let (screen, view) = book.note(CONTROLLER, VIEW);

    assert_eq!(book.retire(CONTROLLER), Some(screen));

    assert_eq!(book.view_address(view), None);
    let (next_screen, next_view) = book.note(CONTROLLER, VIEW);
    assert!(![screen, view].contains(&next_screen));
    assert!(![screen, view].contains(&next_view));
}

#[test]
fn closing_is_raised_only_for_known_controllers() {
    let book = ScreenBook::new();
    book.note(CONTROLLER, VIEW);

    assert_eq!(book.retire(0x9999), None);
    assert!(book.retire(CONTROLLER).is_some());
    assert_eq!(book.retire(CONTROLLER), None);
}

#[test]
fn a_reused_controller_address_gets_a_new_generation() {
    let book = ScreenBook::new();
    let (first, _) = book.note(CONTROLLER, VIEW);
    book.retire(CONTROLLER);

    let (second, _) = book.note(CONTROLLER, VIEW);

    assert!(second > first);
}

#[test]
fn a_view_that_moves_to_another_controller_gets_a_new_view_id() {
    let book = ScreenBook::new();
    let (_, first) = book.note(CONTROLLER, VIEW);

    let (_, second) = book.note(0x3000, VIEW);

    assert_ne!(first, second);
    assert_eq!(book.view_address(first), None);
    assert_eq!(book.view_address(second), Some(VIEW));
}

#[test]
fn retiring_a_controller_retires_only_its_views() {
    let book = ScreenBook::new();
    let (_, mine) = book.note(CONTROLLER, VIEW);
    let (_, other) = book.note(0x3000, 0x4000);

    book.retire(CONTROLLER);

    assert_eq!(book.view_address(mine), None);
    assert_eq!(book.view_address(other), Some(0x4000));
}

#[test]
fn a_controller_gives_its_screen_id_until_it_is_retired() {
    let book = ScreenBook::new();
    assert_eq!(book.screen_of_controller(CONTROLLER), None);
    let (screen, _) = book.note(CONTROLLER, VIEW);

    assert_eq!(book.screen_of_controller(CONTROLLER), Some(screen));
    book.retire(CONTROLLER);

    assert_eq!(book.screen_of_controller(CONTROLLER), None);
}

#[test]
fn a_controller_named_before_its_first_draw_keeps_that_id_at_the_draw() {
    let book = ScreenBook::new();

    let screen = book.note_controller(CONTROLLER);
    assert_ne!(screen, 0);
    assert_eq!(book.note_controller(CONTROLLER), screen);

    let (drawn, _) = book.note(CONTROLLER, VIEW);
    assert_eq!(drawn, screen);
    assert_eq!(book.retire(CONTROLLER), Some(screen));
    assert_ne!(book.note_controller(CONTROLLER), screen);
}

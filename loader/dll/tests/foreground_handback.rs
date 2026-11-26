use amml::ForegroundHandback;

const GAME: u32 = 4100;
const PERSON_WINDOW: usize = 0x1_0010;
const GAME_WINDOW: usize = 0x2_0020;

#[test]
fn a_game_window_in_the_foreground_gives_it_back_to_the_saved_window() {
    let handback = ForegroundHandback::new(true, PERSON_WINDOW, GAME);
    assert_eq!(handback.hand_back_to(GAME_WINDOW, GAME), Some(PERSON_WINDOW));
}

#[test]
fn another_process_in_the_foreground_is_left_alone() {
    let handback = ForegroundHandback::new(true, PERSON_WINDOW, GAME);
    assert_eq!(handback.hand_back_to(0x3_0030, GAME + 1), None);
}

#[test]
fn the_saved_window_in_the_foreground_is_left_alone() {
    let handback = ForegroundHandback::new(true, PERSON_WINDOW, GAME);
    assert_eq!(handback.hand_back_to(PERSON_WINDOW, GAME), None);
}

#[test]
fn without_the_switch_the_game_keeps_the_foreground() {
    let handback = ForegroundHandback::new(false, PERSON_WINDOW, GAME);
    assert_eq!(handback.hand_back_to(GAME_WINDOW, GAME), None);
}

#[test]
fn without_a_saved_window_nothing_is_given_back() {
    let handback = ForegroundHandback::new(true, 0, GAME);
    assert_eq!(handback.hand_back_to(GAME_WINDOW, GAME), None);
}

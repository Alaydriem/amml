use amml_api::{Chord, ChordError, KeyCode, Modifiers};

fn text(input: &str) -> String {
    Chord::parse(input).unwrap().to_string()
}

#[test]
fn chords_parse_the_settings_text() {
    assert_eq!(Chord::parse("Ctrl+LeftBracket"), Ok(Chord { key: KeyCode::LEFT_BRACKET, modifiers: Modifiers::CTRL }));
    assert_eq!(Chord::parse("Ctrl"), Ok(Chord { key: KeyCode::CTRL, modifiers: Modifiers::NONE }));
    assert_eq!(Chord::parse("F3+G"), Ok(Chord { key: KeyCode::G, modifiers: Modifiers::F3 }));
}

#[test]
fn chords_print_the_canonical_settings_text() {
    assert_eq!(text("shift+CTRL+m"), "Ctrl+Shift+M");
    assert_eq!(text("Alt+Shift+Ctrl+F1"), "Ctrl+Shift+Alt+F1");
    assert_eq!(text("f3+ctrl+g"), "Ctrl+F3+G");
    assert_eq!(text("alt+vk_0xe9"), "Alt+VK_0xE9");
}

#[test]
fn text_that_is_not_a_chord_gives_an_error() {
    assert_eq!(Chord::parse("Nope"), Err(ChordError::UnknownKey("Nope".to_string())));
    assert_eq!(Chord::parse(""), Err(ChordError::UnknownKey(String::new())));
    assert_eq!(Chord::parse("Ctrl+"), Err(ChordError::UnknownKey(String::new())));
    assert_eq!(Chord::parse("Ctrl+Banana"), Err(ChordError::UnknownKey("Banana".to_string())));
    assert_eq!(Chord::parse("Ctrl+Ctrl+M"), Err(ChordError::RepeatedModifier("Ctrl".to_string())));
    assert_eq!(Chord::parse("Alt+F1+Ctrl"), Err(ChordError::UnknownModifier("F1".to_string())));
    assert_eq!(Chord::parse("g+f3+ctrl"), Err(ChordError::UnknownModifier("g".to_string())));
    assert_eq!(Chord::parse("VK_0xZZ"), Err(ChordError::UnknownKey("VK_0xZZ".to_string())));
    assert_eq!(Chord::parse("F3"), Err(ChordError::F3IsTheKey));
    assert_eq!(Chord::parse("Ctrl+F3"), Err(ChordError::F3IsTheKey));
}

#[test]
fn a_chord_matches_either_side_of_a_modifier() {
    let chord = Chord::parse("Ctrl+Shift+M").unwrap();

    assert!(chord.matches(KeyCode::M, Modifiers::LCTRL | Modifiers::RSHIFT));
    assert!(chord.matches(KeyCode::M, Modifiers::RCTRL | Modifiers::LSHIFT));
    assert!(chord.matches(KeyCode::M, Modifiers::CTRL | Modifiers::SHIFT));
    assert!(!chord.matches(KeyCode::M, Modifiers::LCTRL));
    assert!(!chord.matches(KeyCode::M, Modifiers::LCTRL | Modifiers::RSHIFT | Modifiers::LALT));
    assert!(!chord.matches(KeyCode::N, Modifiers::LCTRL | Modifiers::RSHIFT));
}

#[test]
fn a_modifier_key_matches_either_side_and_does_not_count_itself() {
    // F3 held, then Shift: the key is Shift, so the Shift bit the press itself sets is not counted.
    let chord = Chord::parse("F3+Shift").unwrap();

    assert!(chord.matches(KeyCode::LSHIFT, Modifiers::LSHIFT.with_f3(true)));
    assert!(chord.matches(KeyCode::RSHIFT, Modifiers::RSHIFT.with_f3(true)));
    assert!(!chord.matches(KeyCode::LSHIFT, Modifiers::LSHIFT));
    assert!(!chord.matches(KeyCode::LSHIFT, (Modifiers::LSHIFT | Modifiers::LCTRL).with_f3(true)));
}

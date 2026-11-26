use amml_api::KeyCode;

#[test]
fn generic_folds_sides() {
    assert_eq!(KeyCode::LSHIFT.generic(), KeyCode::SHIFT);
    assert_eq!(KeyCode::RSHIFT.generic(), KeyCode::SHIFT);
    assert_eq!(KeyCode::RCONTROL.generic(), KeyCode::CTRL);
    assert_eq!(KeyCode::LALT.generic(), KeyCode::ALT);
    assert_eq!(KeyCode::RWIN.generic(), KeyCode::LWIN);
    assert_eq!(KeyCode::W.generic(), KeyCode::W);
}

#[test]
fn names_are_the_settings_text_names() {
    assert_eq!(KeyCode::LEFT_BRACKET.name(), Some("LeftBracket"));
    assert_eq!(KeyCode::CTRL.name(), Some("Ctrl"));
    assert_eq!(KeyCode(0xE9).name(), None);
    assert_eq!(KeyCode::from_name("leftbracket"), Some(KeyCode::LEFT_BRACKET));
    assert_eq!(KeyCode::from_name("vk_0xe9"), Some(KeyCode(0xE9)));
    assert_eq!(KeyCode::from_name("Banana"), None);
    assert_eq!(KeyCode::from_name("VK_0x"), None);
    assert_eq!(KeyCode::from_name("VK_0x100"), None);
}

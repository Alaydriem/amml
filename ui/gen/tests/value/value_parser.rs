use amml_ui_gen::error::GenError;
use amml_ui_gen::model::{TokenEntry, TokenKind};
use amml_ui_gen::value::{TokenValue, ValueParser};

#[test]
fn hex_six() {
    assert_eq!(
        ValueParser::parse(TokenKind::Color, "#8B7CFF").unwrap(),
        TokenValue::Color { r: 0x8B, g: 0x7C, b: 0xFF, a: 255 }
    );
}

#[test]
fn hex_eight() {
    assert_eq!(
        ValueParser::parse(TokenKind::Color, "#8B7CFF80").unwrap(),
        TokenValue::Color { r: 0x8B, g: 0x7C, b: 0xFF, a: 0x80 }
    );
}

#[test]
fn rgb_slash_percent() {
    assert_eq!(
        ValueParser::parse(TokenKind::Color, "rgb(139 124 255 / 18%)").unwrap(),
        TokenValue::Color { r: 139, g: 124, b: 255, a: 46 }
    );
}

#[test]
fn rgb_without_alpha_is_opaque() {
    assert_eq!(
        ValueParser::parse(TokenKind::Color, "rgb(1 2 3)").unwrap(),
        TokenValue::Color { r: 1, g: 2, b: 3, a: 255 }
    );
}

#[test]
fn alias() {
    assert_eq!(
        ValueParser::parse(TokenKind::Color, "var(--color-success)").unwrap(),
        TokenValue::Alias("--color-success".into())
    );
}

#[test]
fn px() {
    assert_eq!(ValueParser::parse(TokenKind::Spacing, "12.5px").unwrap(), TokenValue::Px(12.5));
}

#[test]
fn em() {
    assert_eq!(ValueParser::parse(TokenKind::LetterSpacing, "0.08em").unwrap(), TokenValue::Em(0.08));
}

#[test]
fn percent() {
    assert_eq!(ValueParser::parse(TokenKind::Opacity, "45%").unwrap(), TokenValue::Percent(0.45));
}

#[test]
fn weight() {
    assert_eq!(ValueParser::parse(TokenKind::FontWeight, "600").unwrap(), TokenValue::Number(600.0));
}

#[test]
fn family() {
    assert_eq!(
        ValueParser::parse(TokenKind::FontFamily, "Nordwand Mono").unwrap(),
        TokenValue::Family("Nordwand Mono".into())
    );
}

#[test]
fn color_mix_is_rejected() {
    assert!(matches!(
        ValueParser::parse(TokenKind::Color, "color-mix(in srgb, red, blue)"),
        Err(GenError::BadValue { .. })
    ));
}

#[test]
fn a_unit_that_does_not_fit_the_family_is_rejected() {
    assert!(matches!(ValueParser::parse(TokenKind::Spacing, "45%"), Err(GenError::BadValue { .. })));
    assert!(matches!(ValueParser::parse(TokenKind::FontWeight, "600px"), Err(GenError::BadValue { .. })));
}

#[test]
fn a_weight_is_a_whole_number_from_1_to_1000() {
    assert!(matches!(ValueParser::parse(TokenKind::FontWeight, "550.5"), Err(GenError::BadValue { .. })));
    assert!(matches!(ValueParser::parse(TokenKind::FontWeight, "0"), Err(GenError::BadValue { .. })));
    assert!(matches!(ValueParser::parse(TokenKind::FontWeight, "70000"), Err(GenError::BadValue { .. })));
}

#[test]
fn a_bad_entry_names_its_token() {
    let entry =
        TokenEntry { name: "--color-broken".into(), kind: TokenKind::Color, value: "#12".into(), description: None };
    match ValueParser::parse_entry(&entry) {
        Err(GenError::BadValue { token, raw }) => {
            assert_eq!(token, "--color-broken");
            assert_eq!(raw, "#12");
        }
        other => panic!("expected BadValue, got {other:?}"),
    }
}

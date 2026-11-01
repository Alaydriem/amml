use amml_ui_gen::model::TokenKind;
use amml_ui_gen::naming::TokenName;

#[test]
fn splits_a_colour() {
    assert_eq!(TokenName::split("--color-accent-hover"), Some((TokenKind::Color, "accent-hover".to_owned())));
}

#[test]
fn font_weight_is_not_a_font_family() {
    assert_eq!(TokenName::split("--font-weight-semibold"), Some((TokenKind::FontWeight, "semibold".to_owned())));
    assert_eq!(TokenName::split("--font-ui"), Some((TokenKind::FontFamily, "ui".to_owned())));
}

#[test]
fn every_family_prefix_is_known() {
    let cases = [
        ("--text-body", TokenKind::FontSize),
        ("--tracking-section", TokenKind::LetterSpacing),
        ("--spacing-row", TokenKind::Spacing),
        ("--radius-panel", TokenKind::Radius),
        ("--opacity-soft", TokenKind::Opacity),
        ("--container-menu", TokenKind::Container),
        ("--breakpoint-small", TokenKind::Breakpoint),
        ("--leading-body", TokenKind::LineHeight),
    ];
    for (css, kind) in cases {
        assert_eq!(TokenName::split(css).map(|(found, _)| found), Some(kind), "{css}");
    }
}

#[test]
fn an_unknown_prefix_or_an_empty_rest_is_none() {
    assert_eq!(TokenName::split("--shadow-floating"), None);
    assert_eq!(TokenName::split("--color-"), None);
}

#[test]
fn pascal_and_snake() {
    assert_eq!(TokenName::pascal("scene-1"), "Scene1");
    assert_eq!(TokenName::pascal("accent-hover"), "AccentHover");
    assert_eq!(TokenName::pascal("read-only"), "ReadOnly");
    assert_eq!(TokenName::snake("accent-soft-hover"), "accent_soft_hover");
    assert_eq!(TokenName::snake("scene-1"), "scene_1");
}

#[test]
fn a_rest_that_is_not_an_identifier_is_rejected() {
    assert!(TokenName::is_identifier("accent-hover"));
    assert!(TokenName::is_identifier("scene-1"));
    assert!(!TokenName::is_identifier("2xl"));
    assert!(!TokenName::is_identifier("type"));
    assert!(!TokenName::is_identifier("a--b"));
    assert!(!TokenName::is_identifier("Accent"));
}

#[test]
fn family_module_names() {
    assert_eq!(TokenName::family(TokenKind::Color), "color");
    assert_eq!(TokenName::family(TokenKind::FontSize), "text_size");
    assert_eq!(TokenName::family(TokenKind::LetterSpacing), "tracking");
    assert_eq!(TokenName::family(TokenKind::LineHeight), "line_height");
}

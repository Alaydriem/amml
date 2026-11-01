use amml_ui_gen::model::{TokenFile, TokenKind};

const PAPER_EXPORT: &str = r##"{"tokens":[
  {"name":"--color-accent","type":"color","value":"#8B7CFF","description":"ACCENT"},
  {"name":"--color-shadow","type":"color","value":"rgb(0 0 0 / 59%)"},
  {"name":"--text-body","type":"fontSize","value":"16.5px"},
  {"name":"--leading-body","type":"lineHeight","value":"20px"},
  {"name":"--tracking-section","type":"letterSpacing","value":"0.08em"}
]}"##;

#[test]
fn reads_the_paper_export() {
    let file: TokenFile = serde_json::from_str(PAPER_EXPORT).unwrap();
    assert_eq!(file.tokens.len(), 5);
    assert_eq!(file.tokens[0].name, "--color-accent");
    assert_eq!(file.tokens[0].kind, TokenKind::Color);
    assert_eq!(file.tokens[0].description.as_deref(), Some("ACCENT"));
    assert_eq!(file.tokens[1].description, None);
    assert_eq!(file.tokens[2].kind, TokenKind::FontSize);
    assert_eq!(file.tokens[3].kind, TokenKind::LineHeight);
    assert_eq!(file.tokens[4].kind, TokenKind::LetterSpacing);
}

#[test]
fn an_unknown_type_is_an_error() {
    let bad = r#"{"tokens":[{"name":"--shadow-x","type":"shadow","value":"0 0 1px red"}]}"#;
    assert!(serde_json::from_str::<TokenFile>(bad).is_err());
}

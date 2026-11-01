use amml_i18n::service::LocaleSetting;

#[test]
fn from_code_maps_the_three_forms() {
    assert_eq!(LocaleSetting::from_code(""), LocaleSetting::English);
    assert_eq!(LocaleSetting::from_code("en_XA"), LocaleSetting::Pseudo);
    assert_eq!(LocaleSetting::from_code("pt_BR"), LocaleSetting::Locale("pt_BR".to_owned()));
    assert_eq!(LocaleSetting::from_code("  "), LocaleSetting::English);
    assert_eq!(LocaleSetting::from_code(" ru "), LocaleSetting::Locale("ru".to_owned()));
}

#[test]
fn code_round_trips() {
    for code in ["", "en_XA", "ru"] {
        assert_eq!(LocaleSetting::from_code(code).code(), code);
    }
}

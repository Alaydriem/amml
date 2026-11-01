use amml_i18n::negotiate::LocaleNegotiator;

fn available() -> Vec<String> {
    vec!["de".into(), "pt_BR".into(), "ru".into(), "zh_CN".into()]
}

#[test]
fn exact_region_match_wins_over_bare_language() {
    assert_eq!(LocaleNegotiator::resolve("pt_BR", &available()), Some("pt_BR".to_string()));
}

#[test]
fn hyphenated_request_matches_underscored_catalog() {
    assert_eq!(LocaleNegotiator::resolve("pt-BR", &available()), Some("pt_BR".to_string()));
}

#[test]
fn region_request_falls_back_to_bare_language() {
    assert_eq!(LocaleNegotiator::resolve("ru-RU", &available()), Some("ru".to_string()));
}

#[test]
fn bare_request_does_not_match_a_region_only_catalog_entry() {
    assert_eq!(LocaleNegotiator::resolve("pt", &available()), None);
}

#[test]
fn unknown_locale_resolves_to_nothing_so_the_caller_uses_english() {
    assert_eq!(LocaleNegotiator::resolve("is-IS", &available()), None);
}

#[test]
fn matching_is_case_insensitive_because_os_locales_vary() {
    assert_eq!(LocaleNegotiator::resolve("ZH-cn", &available()), Some("zh_CN".to_string()));
}

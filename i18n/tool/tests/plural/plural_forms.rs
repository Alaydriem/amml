use amml_i18n_tool::plural::PluralForms;

const RUSSIAN: &str =
    "nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2);";
const POLISH: &str = "nplurals=3; plural=(n==1 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2);";

fn categories(locale: &str, header: &str) -> Vec<String> {
    PluralForms::check(locale, header).unwrap_or_else(|error| panic!("{locale}: {error}"))
}

#[test]
fn english_maps_the_two_gettext_forms_onto_cldr_one_other() {
    assert_eq!(categories("en", "nplurals=2; plural=(n != 1);"), ["one", "other"]);
}

#[test]
fn japanese_has_a_single_form_and_it_is_cldr_other() {
    assert_eq!(categories("ja", "nplurals=1; plural=0;"), ["other"]);
}

#[test]
fn russian_resolves_three_gettext_forms_even_though_cldr_names_four_categories() {
    assert_eq!(categories("ru", RUSSIAN), ["one", "few", "many"]);
}

#[test]
fn polish_resolves_in_gettext_index_order() {
    assert_eq!(categories("pl", POLISH), ["one", "few", "many"]);
}

#[test]
fn french_treats_one_as_singular_despite_its_zero_is_singular_rule() {
    assert_eq!(categories("fr", "nplurals=2; plural=(n > 1);"), ["one", "other"]);
}

#[test]
fn underscored_catalog_locales_are_accepted() {
    assert_eq!(categories("pt_BR", "nplurals=2; plural=(n > 1);"), ["one", "other"]);
}

#[test]
fn an_expression_disagreeing_with_the_locale_rules_is_rejected() {
    let error = PluralForms::check("en", "nplurals=3; plural=(n%3);").err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("maps to both")));
}

#[test]
fn a_header_declaring_more_forms_than_the_expression_produces_is_rejected() {
    let error = PluralForms::check("en", "nplurals=4; plural=(n != 1);").err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("never selected")));
}

// Review finding 1: the old Hebrew header has four forms but CLDR has three categories, so
// two forms would share `other` and the runtime would always pick the first one.
#[test]
fn two_forms_with_the_same_category_are_rejected() {
    let hebrew = "nplurals=4; plural=(n==1 ? 0 : n==2 ? 1 : n>10 && n%10==0 ? 2 : 3);";
    let error = PluralForms::check("he", hebrew).err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("both map to")), "duplicate category accepted");
}

// A valid tag with no CLDR data gets ICU's root rules, where every count is `other`.
#[test]
fn a_locale_without_cldr_data_is_rejected_for_a_two_form_header() {
    assert!(PluralForms::check("xx", "nplurals=2; plural=(n != 1);").is_err());
}

#[test]
fn an_unparseable_header_is_rejected_rather_than_guessed_at() {
    let error = PluralForms::check("en", "garbage").err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("Plural-Forms")));
}

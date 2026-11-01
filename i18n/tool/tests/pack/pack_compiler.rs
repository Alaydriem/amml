use amml_i18n::pack::PackEntry;
use amml_i18n_tool::pack::PackCompiler;
use amml_i18n_tool::po::{PoCatalog, PoReader};

const RUSSIAN: &str = r#"msgid ""
msgstr ""
"Plural-Forms: nplurals=3; plural=(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2);\n"

msgid "Sign In Again"
msgstr "Войти снова"

msgid "Never Translated"
msgstr ""

msgid "{count} player nearby"
msgid_plural "{count} players nearby"
msgstr[0] "{count} игрок"
msgstr[1] "{count} игрока"
msgstr[2] "{count} игроков"

msgctxt "audio"
msgid "Output"
msgstr "Выход"
"#;

fn russian() -> PoCatalog {
    PoReader::parse(RUSSIAN).unwrap_or_else(|error| panic!("{error}"))
}

fn with(extra: &str) -> PoCatalog {
    PoReader::parse(&format!("{RUSSIAN}\n{extra}")).unwrap_or_else(|error| panic!("{error}"))
}

fn compile(catalog: &PoCatalog) -> amml_i18n::pack::LanguagePack {
    PackCompiler::compile(catalog, "ru").unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn plural_categories_are_resolved_onto_the_pack() {
    assert_eq!(compile(&russian()).plural, ["one", "few", "many"]);
}

#[test]
fn a_singular_message_compiles_to_a_bare_string() {
    assert_eq!(compile(&russian()).m.get("Sign In Again"), Some(&PackEntry::One("Войти снова".into())));
}

#[test]
fn a_plural_message_compiles_to_the_ordered_form_array() {
    let forms = ["{count} игрок", "{count} игрока", "{count} игроков"].map(String::from).to_vec();
    assert_eq!(compile(&russian()).m.get("{count} player nearby"), Some(&PackEntry::Many(forms)));
}

#[test]
fn context_is_folded_into_the_key_with_the_eot_separator() {
    assert_eq!(compile(&russian()).m.get("audio\u{4}Output"), Some(&PackEntry::One("Выход".into())));
}

#[test]
fn an_untranslated_message_is_omitted_so_lookup_falls_through_to_english() {
    assert!(!compile(&russian()).m.contains_key("Never Translated"));
    assert!(!compile(&russian()).m.contains_key(""));
}

#[test]
fn coverage_counts_translated_messages_against_the_total() {
    let coverage = PackCompiler::coverage(&russian());
    assert_eq!((coverage.translated, coverage.total), (3, 4));
}

#[test]
fn a_plural_entry_missing_one_of_its_forms_does_not_count_as_translated() {
    let catalog = with(
        "msgid \"{count} mod\"\nmsgid_plural \"{count} mods\"\nmsgstr[0] \"мод\"\nmsgstr[1] \"\"\nmsgstr[2] \"модов\"\n",
    );
    let coverage = PackCompiler::coverage(&catalog);
    assert_eq!((coverage.translated, coverage.total), (3, 5));
    assert!(!compile(&catalog).m.contains_key("{count} mod"));
}

#[test]
fn compiler_rejects_placeholder_mismatch() {
    let catalog = with("msgid \"Hello, {name}\"\nmsgstr \"Привет, {nom}\"\n");
    let error = PackCompiler::compile(&catalog, "ru").err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("Hello, {name}")), "error must name the msgid");
}

#[test]
fn compiler_rejects_a_dropped_placeholder() {
    let catalog = with("msgid \"Hello, {name}\"\nmsgstr \"Привет\"\n");
    assert!(PackCompiler::compile(&catalog, "ru").is_err());
}

// Some languages spell out "one" for the singular form, so a plural form may leave out
// {count}. Every other placeholder must stay.
#[test]
fn a_plural_form_may_leave_out_count_but_nothing_else() {
    let ok = with(
        "msgid \"{count} mod\"\nmsgid_plural \"{count} mods\"\nmsgstr[0] \"один мод\"\nmsgstr[1] \"{count} мода\"\nmsgstr[2] \"{count} модов\"\n",
    );
    assert!(PackCompiler::compile(&ok, "ru").is_ok());
    let bad = with(
        "msgid \"{count} mod in {pack}\"\nmsgid_plural \"{count} mods in {pack}\"\nmsgstr[0] \"{count} мод\"\nmsgstr[1] \"{count} мода в {pack}\"\nmsgstr[2] \"{count} модов в {pack}\"\n",
    );
    assert!(PackCompiler::compile(&bad, "ru").is_err());
}

// Review finding 3: two lines for a three-form locale would show the wrong form for 5.
#[test]
fn a_plural_entry_with_fewer_forms_than_the_header_is_not_translated() {
    let catalog = with(
        "msgid \"{count} mod\"\nmsgid_plural \"{count} mods\"\nmsgstr[0] \"{count} мод\"\nmsgstr[1] \"{count} мода\"\n",
    );
    let coverage = PackCompiler::coverage(&catalog);
    assert_eq!((coverage.translated, coverage.total), (3, 5));
    assert!(!compile(&catalog).m.contains_key("{count} mod"));
}

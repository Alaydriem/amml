use amml_i18n_tool::po::PoReader;

const PO: &str = r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"
"Plural-Forms: nplurals=2; plural=(n > 1);\n"

msgid "Save"
msgstr "Enregistrer"

msgctxt "audio"
msgid "Output"
msgstr "Sortie"

#, fuzzy
msgid "Close"
msgstr "Fermer"

msgid "{count} mod"
msgid_plural "{count} mods"
msgstr[0] "{count} mod"
msgstr[1] "{count} mods"
"#;

#[test]
fn the_header_is_not_a_message() {
    let catalog = PoReader::parse(PO).unwrap_or_else(|error| panic!("{error}"));
    assert!(catalog.messages.iter().all(|message| !message.msgid.is_empty()));
    assert_eq!(catalog.messages.len(), 4);
}

#[test]
fn context_plural_and_forms_are_read() {
    let catalog = PoReader::parse(PO).unwrap_or_else(|error| panic!("{error}"));
    let output = catalog.messages.iter().find(|message| message.msgid == "Output");
    assert_eq!(output.and_then(|message| message.context.as_deref()), Some("audio"));
    let mods = catalog.messages.iter().find(|message| message.msgid == "{count} mod");
    assert_eq!(mods.and_then(|message| message.msgid_plural.as_deref()), Some("{count} mods"));
    assert_eq!(mods.map(|message| message.msgstr.len()), Some(2));
}

// A fuzzy entry is a guess the translator has not confirmed; gettext does not ship it.
#[test]
fn a_fuzzy_entry_is_not_translated() {
    let catalog = PoReader::parse(PO).unwrap_or_else(|error| panic!("{error}"));
    let close = catalog.messages.iter().find(|message| message.msgid == "Close");
    assert_eq!(close.map(|message| message.is_translated()), Some(false));
}

#[test]
fn the_plural_forms_header_is_kept() {
    let catalog = PoReader::parse(PO).unwrap_or_else(|error| panic!("{error}"));
    assert!(catalog.plural_forms.contains("nplurals=2"));
    assert!(catalog.plural_forms.contains("(n > 1)"));
}

// Review finding 2: polib panics on a stray quote. A typo must be an error that names the
// file, not a crash part way through the locales.
#[test]
fn a_po_typo_is_an_error_not_a_crash() {
    for text in ["msgid \"", "msgid \"a\"\nmsgstr \"b\"\n\"\n"] {
        assert!(PoReader::parse(text).is_err(), "{text:?} was accepted");
    }
    let directory = tempfile::TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    let path = directory.path().join("broken.po");
    std::fs::write(&path, "msgid \"").unwrap_or_else(|error| panic!("{error}"));
    let error = PoReader::read(&path).err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("broken.po")));
}

#[test]
fn a_missing_file_names_its_path() {
    let error = PoReader::read(std::path::Path::new("no/such/file.po")).err().map(|e| e.to_string());
    assert!(error.is_some_and(|e| e.contains("file.po")));
}

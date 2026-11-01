use std::path::{Path, PathBuf};

use amml_i18n::localize::Localizer;
use amml_i18n::message::{Domain, Message};
use amml_i18n::service::{LocaleSetting, LocalizationService, PackSource};
use tempfile::TempDir;

use crate::fixture::SampleMessage;

const SAMPLE: Domain = SampleMessage::DOMAIN;
const OTHER_MOD: Domain = Domain::new("other-mod");

const RUSSIAN: &[u8] = r#"{"v":1,"locale":"ru","plural":["one","few","many"],
    "m":{"Sign In Again":"Войти снова","{count} player nearby":["игрок","игрока","игроков"]}}"#
    .as_bytes();
const PSEUDO: &[u8] =
    r#"{"v":1,"locale":"en_XA","plural":["one","other"],"m":{"Sign In Again":"[Šîĝñ Îñ Åĝåîñ ~~~~]"}}"#.as_bytes();
const BROKEN: &[u8] = b"{ this is not json";

static EMBEDDED: &[(&str, &[u8])] = &[("en_XA", PSEUDO), ("ru", RUSSIAN)];
static MALFORMED: &[(&str, &[u8])] = &[("ru", BROKEN)];

fn embedded() -> LocalizationService {
    LocalizationService::new(vec![(SAMPLE, PackSource::Embedded(EMBEDDED))])
}

fn sign_in(localizer: &Localizer) -> &str {
    localizer.table::<SampleMessage>().text(SampleMessage::SignInAgain)
}

fn directory() -> TempDir {
    let directory = TempDir::new().unwrap_or_else(|error| panic!("temp dir: {error}"));
    for (name, body) in [
        ("ru.json", RUSSIAN),
        ("de.json", br#"{"v":1,"locale":"de","plural":["one","other"],"m":{}}"#),
        ("notes.txt", b"not a pack"),
    ] {
        std::fs::write(directory.path().join(name), body).unwrap_or_else(|error| panic!("write {name}: {error}"));
    }
    directory
}

fn from_directory(path: &Path) -> LocalizationService {
    LocalizationService::new(vec![(SAMPLE, PackSource::Directory(path.to_path_buf()))])
}

#[test]
fn english_setting_needs_no_pack() {
    let localizer = embedded().load(&LocaleSetting::English);
    assert_eq!(localizer.locale(), None);
    assert_eq!(sign_in(&localizer), "Sign In Again");
}

#[test]
fn embedded_pack_loads() {
    let localizer = embedded().load(&LocaleSetting::Locale("ru".into()));
    assert_eq!(sign_in(&localizer), "Войти снова");
    assert_eq!(localizer.table::<SampleMessage>().plural(SampleMessage::PlayersNearby, 3), "игрока");
}

// A pack is content the mod did not author. A malformed one must not stop the UI, so it
// degrades to English.
#[test]
fn malformed_pack_is_english() {
    let service = LocalizationService::new(vec![(SAMPLE, PackSource::Embedded(MALFORMED))]);
    let localizer = service.load(&LocaleSetting::Locale("ru".into()));
    assert_eq!(sign_in(&localizer), "Sign In Again");
}

#[test]
fn negotiates_ru_ru_to_ru() {
    let localizer = embedded().load(&LocaleSetting::Locale("ru-RU".into()));
    assert_eq!(sign_in(&localizer), "Войти снова");
}

#[test]
fn a_locale_that_is_not_shipped_is_english() {
    let localizer = embedded().load(&LocaleSetting::Locale("is-IS".into()));
    assert_eq!(localizer.locale(), None);
    assert_eq!(sign_in(&localizer), "Sign In Again");
}

#[test]
fn pseudo_setting_loads_en_xa() {
    let localizer = embedded().load(&LocaleSetting::Pseudo);
    assert_eq!(sign_in(&localizer), "[Šîĝñ Îñ Åĝåîñ ~~~~]");
}

#[test]
fn a_pack_serves_only_its_own_domain() {
    let service = LocalizationService::new(vec![(OTHER_MOD, PackSource::Embedded(EMBEDDED))]);
    let localizer = service.load(&LocaleSetting::Locale("ru".into()));
    assert!(localizer.has_pack(OTHER_MOD));
    assert_eq!(sign_in(&localizer), "Sign In Again");
}

#[test]
fn only_json_files_are_offered_as_locales() {
    let packs = directory();
    assert_eq!(from_directory(packs.path()).available(SAMPLE), vec!["de".to_string(), "ru".to_string()]);
}

#[test]
fn a_missing_directory_offers_nothing_rather_than_failing() {
    let service = from_directory(&PathBuf::from("does/not/exist"));
    assert!(service.available(SAMPLE).is_empty());
    assert_eq!(sign_in(&service.load(&LocaleSetting::Locale("ru".into()))), "Sign In Again");
}

#[test]
fn a_directory_pack_loads() {
    let packs = directory();
    let localizer = from_directory(packs.path()).load(&LocaleSetting::Locale("ru_RU".into()));
    assert_eq!(sign_in(&localizer), "Войти снова");
}

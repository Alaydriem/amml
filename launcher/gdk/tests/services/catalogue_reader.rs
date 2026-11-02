use crate::support::{FakeServer, Route};
use amml_gdk::{CatalogueReader, Channel, GameVersion, GdkError};

const SAMPLE: &str = r#"{
  "release": {
    "1.21.120.4": ["https://a.example/x/Microsoft.MinecraftUWP_1.21.12004.0_x64__8wekyb3d8bbwe.msixvc", "https://b.example/Microsoft.MinecraftUWP_1.21.12004.0_x64__8wekyb3d8bbwe.msixvc"],
    "latest": ["https://a.example/ignored.msixvc"],
    "1.26.60.5": ["https://a.example/x/Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe.msixvc", "https://b.example/Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe.msixvc"]
  },
  "preview": {
    "1.26.70.22": ["https://a.example/Microsoft.MinecraftWindowsBeta_1.26.7022.0_x64__8wekyb3d8bbwe.msixvc", "https://b.example/Microsoft.MinecraftWindowsBeta_1.26.7022.0_x64__8wekyb3d8bbwe.msixvc"]
  }
}"#;

fn v(raw: &str) -> GameVersion {
    GameVersion::parse(raw).unwrap()
}

#[test]
fn splits_release_and_preview_newest_first() {
    let catalogue = CatalogueReader::parse(SAMPLE).unwrap();
    let release: Vec<_> = catalogue.entries(Channel::Release).iter().map(|e| e.version).collect();
    assert_eq!(release, vec![v("1.26.60.5"), v("1.21.120.4")]);
    let preview = catalogue.entries(Channel::Preview);
    assert_eq!(preview.len(), 1);
    assert_eq!(preview[0].version, v("1.26.70.22"));
    assert_eq!(catalogue.entries(Channel::Release)[0].urls.len(), 2);
}

#[test]
fn finds_entry_by_package_full_name() {
    let catalogue = CatalogueReader::parse(SAMPLE).unwrap();
    let entry = catalogue.find_by_full_name("Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe").unwrap();
    assert_eq!(entry.version, v("1.26.60.5"));
    assert!(catalogue.find_by_full_name("Microsoft.MinecraftUWP_9.9.9999.0_x64__8wekyb3d8bbwe").is_none());
    let preview = catalogue.find_by_full_name("Microsoft.MinecraftWindowsBeta_1.26.7022.0_x64__8wekyb3d8bbwe").unwrap();
    assert_eq!(preview.channel, Channel::Preview);
}

#[test]
fn find_matches_version_within_channel() {
    let catalogue = CatalogueReader::parse(SAMPLE).unwrap();
    assert!(catalogue.find(&v("1.26.60.5"), Channel::Release).is_some());
    assert!(catalogue.find(&v("1.26.60.5"), Channel::Preview).is_none());
}

#[test]
fn missing_section_is_bad_catalogue() {
    assert!(matches!(CatalogueReader::parse(r#"{"release":{}}"#), Err(GdkError::BadCatalogue("preview"))));
}

#[test]
fn bad_version_key_is_skipped_not_fatal() {
    let catalogue = CatalogueReader::parse(SAMPLE).unwrap();
    assert_eq!(catalogue.entries(Channel::Release).len(), 2);
}

#[test]
fn fetch_reads_the_catalogue_from_a_server() {
    let server = FakeServer::start(vec![("/urls.json", Route::new(SAMPLE.as_bytes().to_vec()))]);
    let catalogue = CatalogueReader::fetch_from(&FakeServer::agent(), &server.url("/urls.json")).unwrap();
    assert_eq!(catalogue.entries(Channel::Release).len(), 2);
}

#[test]
fn fetch_of_a_missing_catalogue_is_a_download_error() {
    let server = FakeServer::start(vec![]);
    let result = CatalogueReader::fetch_from(&FakeServer::agent(), &server.url("/urls.json"));
    assert!(matches!(result, Err(GdkError::Download(_))));
}

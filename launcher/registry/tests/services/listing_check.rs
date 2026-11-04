use amml_registry::{Listing, ListingCheck, RegistryError};

use crate::support::Samples;

fn rejected_field(listing: &Listing) -> &'static str {
    match ListingCheck::check(listing) {
        Err(RegistryError::Listing { field, .. }) => field,
        other => panic!("expected a listing error, got {other:?}"),
    }
}

fn with(change: impl FnOnce(&mut Listing)) -> Listing {
    let mut listing = Samples::listing("Zoom");
    change(&mut listing);
    listing
}

#[test]
fn a_full_listing_passes() {
    let listing = with(|l| {
        l.description = Some("First paragraph.\n\nSecond paragraph.".to_string());
        l.homepage = Some("https://github.com/alaydriem/amml-zoom#readme".to_string());
        l.icon = Some("icon.png".to_string());
        l.tags = vec!["map".to_string(), "hud-2".to_string()];
        l.license = "LicenseRef-Proprietary".to_string();
    });
    ListingCheck::check(&listing).unwrap();
}

#[test]
fn title_rules() {
    assert_eq!(rejected_field(&with(|l| l.title = String::new())), "title");
    assert_eq!(rejected_field(&with(|l| l.title = "x".repeat(41))), "title");
    assert_eq!(rejected_field(&with(|l| l.title = "two\nlines".to_string())), "title");
    ListingCheck::check(&with(|l| l.title = "x".repeat(40))).unwrap();
}

#[test]
fn summary_rules() {
    assert_eq!(rejected_field(&with(|l| l.summary = String::new())), "summary");
    assert_eq!(rejected_field(&with(|l| l.summary = "x".repeat(121))), "summary");
    assert_eq!(rejected_field(&with(|l| l.summary = "a\nb".to_string())), "summary");
    ListingCheck::check(&with(|l| l.summary = "x".repeat(120))).unwrap();
}

#[test]
fn length_counts_characters_not_bytes() {
    ListingCheck::check(&with(|l| l.title = "é".repeat(40))).unwrap();
}

#[test]
fn description_rules() {
    assert_eq!(rejected_field(&with(|l| l.description = Some("x".repeat(2001)))), "description");
    assert_eq!(rejected_field(&with(|l| l.description = Some("bad\u{7}bell".to_string()))), "description");
    assert_eq!(rejected_field(&with(|l| l.description = Some("<b>bold</b>".to_string()))), "description");
    ListingCheck::check(&with(|l| l.description = Some("a > b and 2 < 3".to_string()))).unwrap();
}

#[test]
fn author_rules() {
    assert_eq!(rejected_field(&with(|l| l.authors = Vec::new())), "authors");
    assert_eq!(rejected_field(&with(|l| l.authors = vec!["a".to_string(); 9])), "authors");
    assert_eq!(rejected_field(&with(|l| l.authors = vec![String::new()])), "authors");
    assert_eq!(rejected_field(&with(|l| l.authors = vec!["x".repeat(65)])), "authors");
}

#[test]
fn license_must_look_like_an_spdx_id() {
    assert_eq!(rejected_field(&with(|l| l.license = String::new())), "license");
    assert_eq!(rejected_field(&with(|l| l.license = "BSD 3 clause!".to_string())), "license");
    ListingCheck::check(&with(|l| l.license = "MIT OR Apache-2.0".to_string())).unwrap();
}

#[test]
fn url_rules() {
    assert_eq!(rejected_field(&with(|l| l.source = "http://example.com".to_string())), "source");
    assert_eq!(rejected_field(&with(|l| l.source = String::new())), "source");
    assert_eq!(rejected_field(&with(|l| l.source = "https://a b".to_string())), "source");
    assert_eq!(rejected_field(&with(|l| l.homepage = Some("ftp://x".to_string()))), "homepage");
}

#[test]
fn tag_rules() {
    assert_eq!(rejected_field(&with(|l| l.tags = vec!["Map".to_string()])), "tags");
    assert_eq!(rejected_field(&with(|l| l.tags = vec!["a b".to_string()])), "tags");
    assert_eq!(rejected_field(&with(|l| l.tags = vec!["x".repeat(25)])), "tags");
    assert_eq!(rejected_field(&with(|l| l.tags = vec![String::new()])), "tags");
    assert_eq!(rejected_field(&with(|l| l.tags = (0..9).map(|n| format!("t{n}")).collect())), "tags");
}

#[test]
fn icon_path_rules() {
    assert_eq!(rejected_field(&with(|l| l.icon = Some("../icon.png".to_string()))), "icon");
    assert_eq!(rejected_field(&with(|l| l.icon = Some("/icon.png".to_string()))), "icon");
    assert_eq!(rejected_field(&with(|l| l.icon = Some("C:\\icon.png".to_string()))), "icon");
    assert_eq!(rejected_field(&with(|l| l.icon = Some("icon.jpg".to_string()))), "icon");
}

fn png(width: u32, height: u32, total: usize) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.resize(total, 0);
    bytes
}

#[test]
fn icon_file_rules() {
    ListingCheck::check_icon(&png(128, 128, 2000)).unwrap();
    ListingCheck::check_icon(&png(128, 128, 64 * 1024)).unwrap();
    for bad in [png(128, 128, 64 * 1024 + 1), png(64, 128, 2000), png(128, 127, 2000), b"not a png at all, no".to_vec()]
    {
        match ListingCheck::check_icon(&bad) {
            Err(RegistryError::Listing { field, .. }) => assert_eq!(field, "icon"),
            other => panic!("{other:?}"),
        }
    }
}

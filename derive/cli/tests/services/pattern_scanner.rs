use amml_derive::testing::ImageFixture;
use amml_derive::{DeriveError, Pattern, PatternScanner};

const BYTES_A: [u8; 7] = [0x48, 0x8B, 0x05, 0x11, 0x22, 0x33, 0x44];
const BYTES_B: [u8; 7] = [0x48, 0x8B, 0x05, 0x99, 0x88, 0x77, 0x66];

#[test]
fn wildcards_match_any_byte_and_all_matches_are_returned_in_order() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let a = fx.put_text(&BYTES_A);
    let b = fx.put_text(&BYTES_B);
    let p = Pattern::parse("48 8b 05 ? ? ? ??").unwrap();
    assert_eq!(PatternScanner::find_all(&fx.image(), &p), vec![a, b]);
}

#[test]
fn leading_wildcards_resolve_to_the_pattern_start() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let a = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD]);
    let p = Pattern::parse("? ? CC dd").unwrap();
    assert_eq!(PatternScanner::find_all(&fx.image(), &p), vec![a]);
}

#[test]
fn pattern_does_not_match_in_non_executable_sections() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    fx.put_rdata(&BYTES_A);
    fx.put_data(&BYTES_B);
    let p = Pattern::parse("48 8b 05 ? ? ? ?").unwrap();
    assert!(PatternScanner::find_all(&fx.image(), &p).is_empty());
}

#[test]
fn find_in_limits_matches_to_the_range() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let first = fx.put_text(&BYTES_A);
    let second = fx.put_text(&BYTES_B);
    let p = Pattern::parse("48 8b 05 ? ? ? ?").unwrap();
    assert_eq!(PatternScanner::find_in(&fx.image(), &p, (second, second + 16)), vec![second]);
    assert_eq!(PatternScanner::find_in(&fx.image(), &p, (first, second)), vec![first]);
}

#[test]
fn bad_tokens_are_named() {
    assert!(matches!(Pattern::parse("48 8G"), Err(DeriveError::BadPattern { token }) if token == "8G"));
    assert!(matches!(Pattern::parse("48 123"), Err(DeriveError::BadPattern { token }) if token == "123"));
    assert!(matches!(Pattern::parse("? ??"), Err(DeriveError::BadPattern { .. })));
}

#[test]
fn overlapping_matches_are_all_returned() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let a = fx.put_text(&[0xCC, 0xCC, 0xCC, 0xCC]);
    let p = Pattern::parse("CC CC ?").unwrap();
    assert_eq!(PatternScanner::find_all(&fx.image(), &p), vec![a, a + 1]);
}

#[test]
fn a_match_that_starts_inside_the_range_may_run_past_its_end() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let at = fx.put_text(&BYTES_A);
    let p = Pattern::parse("48 8b 05 ? ? ? ?").unwrap();
    assert_eq!(PatternScanner::find_in(&fx.image(), &p, (at, at + 2)), vec![at]);
    assert!(PatternScanner::find_in(&fx.image(), &p, (at + 1, at + 8)).is_empty());
}

#[test]
fn a_range_outside_every_executable_section_finds_nothing() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    fx.put_text(&BYTES_A);
    let p = Pattern::parse("48 8b 05 ? ? ? ?").unwrap();
    assert!(PatternScanner::find_in(&fx.image(), &p, (0, 0x1000)).is_empty());
    assert!(PatternScanner::find_in(&fx.image(), &p, (0x20_0000, 0x30_0000)).is_empty());
}

#[test]
fn a_leading_wildcard_pattern_at_the_range_start_is_found() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let at = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xDD]);
    let p = Pattern::parse("? ? CC DD").unwrap();
    assert_eq!(PatternScanner::find_in(&fx.image(), &p, (at, at + 1)), vec![at]);
    assert!(PatternScanner::find_in(&fx.image(), &p, (at + 1, at + 4)).is_empty());
}

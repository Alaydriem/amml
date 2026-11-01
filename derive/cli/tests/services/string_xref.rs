use amml_derive::testing::ImageFixture;
use amml_derive::{FunctionTable, Image, Section, StepError, StringXref};

const BASE: u64 = 0x1_4000_0000;
const TEXT: u32 = 0x1000;

struct Probe;

impl Probe {
    fn using(image: &Image, text: &str) -> Result<Vec<u32>, StepError> {
        StringXref::new(image).functions_using(image, text)
    }
}

struct Code;

impl Code {
    // `lea rax, [rip+disp]` placed at `at`, loading `target`
    fn lea(at: u32, target: u32) -> [u8; 7] {
        let disp = target.wrapping_sub(at + 7).to_le_bytes();
        [0x48, 0x8D, 0x05, disp[0], disp[1], disp[2], disp[3]]
    }

    // `lea` instructions at consecutive offsets from `start`, then `ret`
    fn function(start: u32, targets: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (n, target) in targets.iter().enumerate() {
            bytes.extend_from_slice(&Self::lea(start + 7 * n as u32, *target));
        }
        bytes.push(0xC3);
        bytes
    }
}

#[test]
fn returns_each_function_that_loads_the_string_once() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"persona.emote.chat_message\0");
    let a = TEXT;
    assert_eq!(fx.put_text(&Code::function(a, &[s, s])), a);
    let b = TEXT + 16;
    assert_eq!(fx.put_text(&Code::function(b, &[s])), b);
    fx.put_function(a, a + 15);
    fx.put_function(b, b + 8);
    assert_eq!(Probe::using(&fx.image(), "persona.emote.chat_message").unwrap(), vec![a, b]);
}

#[test]
fn prefix_of_a_longer_string_does_not_count() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"abcdef\0");
    fx.put_text(&Code::function(TEXT, &[s]));
    fx.put_function(TEXT, TEXT + 8);
    assert!(matches!(
        Probe::using(&fx.image(), "abc"),
        Err(StepError::StringNotFound(text)) if text == "abc"
    ));
}

#[test]
fn suffix_of_a_longer_string_does_not_count() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"xabc\0");
    fx.put_text(&Code::function(TEXT, &[s + 1]));
    fx.put_function(TEXT, TEXT + 8);
    assert!(matches!(Probe::using(&fx.image(), "abc"), Err(StepError::StringNotFound(_))));
}

#[test]
fn missing_string_is_not_found() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_rdata(b"something\0");
    fx.put_text(&[0xC3]);
    assert!(matches!(Probe::using(&fx.image(), "absent"), Err(StepError::StringNotFound(_))));
}

#[test]
fn every_copy_of_the_string_counts_as_a_target() {
    let mut fx = ImageFixture::new(BASE);
    let first = fx.put_rdata(b"dup\0");
    let second = fx.put_data(b"dup\0");
    let a = TEXT;
    fx.put_text(&Code::function(a, &[first]));
    let b = TEXT + 16;
    fx.put_text(&Code::function(b, &[second]));
    fx.put_function(a, a + 8);
    fx.put_function(b, b + 8);
    assert_eq!(Probe::using(&fx.image(), "dup").unwrap(), vec![a, b]);
}

#[test]
fn lea_in_a_chained_fragment_names_the_primary() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"cold\0");
    let primary = TEXT;
    fx.put_text(&[0xC3]);
    let fragment = TEXT + 16;
    fx.put_text(&Code::function(fragment, &[s]));
    let image = fx.image();
    let table = FunctionTable::with_fragments(vec![(primary, primary + 1)], vec![(fragment, fragment + 8, primary)]);
    let sections: Vec<Section> = image.sections().to_vec();
    let image = Image::new(image.image_base(), sections, table, image.build());
    assert_eq!(Probe::using(&image, "cold").unwrap(), vec![primary]);
}

#[test]
fn lea_outside_every_function_is_ignored() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"orphan\0");
    fx.put_text(&Code::function(TEXT, &[s]));
    assert!(Probe::using(&fx.image(), "orphan").unwrap().is_empty());
}

#[test]
fn lea_with_a_different_target_is_ignored() {
    let mut fx = ImageFixture::new(BASE);
    let s = fx.put_rdata(b"wanted\0");
    fx.put_text(&Code::function(TEXT, &[s + 1]));
    fx.put_function(TEXT, TEXT + 8);
    assert!(Probe::using(&fx.image(), "wanted").unwrap().is_empty());
}

#[test]
fn strings_used_by_lists_leas_in_function_range() {
    let mut fx = ImageFixture::new(BASE);
    let first = fx.put_rdata(b"Alpha::one\0");
    let second = fx.put_rdata(b"Beta::two(int)\0");
    let a = TEXT;
    // loads the second string, then the first one twice
    assert_eq!(fx.put_text(&Code::function(a, &[second, first, first])), a);
    let b = TEXT + 32;
    assert_eq!(fx.put_text(&Code::function(b, &[first])), b);
    fx.put_function(a, a + 22);
    fx.put_function(b, b + 8);
    let image = fx.image();
    let xrefs = StringXref::new(&image);
    assert_eq!(xrefs.strings_used_by(&image, a), vec!["Alpha::one".to_string(), "Beta::two(int)".to_string()]);
    assert_eq!(xrefs.strings_used_by(&image, b), vec!["Alpha::one".to_string()]);
}

#[test]
fn strings_used_by_skips_short_and_binary_data() {
    let mut fx = ImageFixture::new(BASE);
    let short = fx.put_rdata(b"abc\0");
    let binary = fx.put_rdata(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x00]);
    fx.put_text(&Code::function(TEXT, &[short, binary]));
    fx.put_function(TEXT, TEXT + 15);
    let image = fx.image();
    assert!(StringXref::new(&image).strings_used_by(&image, TEXT).is_empty());
}

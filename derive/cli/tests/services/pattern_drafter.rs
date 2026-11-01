use amml_derive::testing::ImageFixture;
use amml_derive::{Image, PatternDrafter, StepError};

const BASE: u64 = 0x1_4000_0000;

// mov rax,[rcx+1A8h]; call rel32; test al,al; ret; returns the image and the function start
fn image() -> (Image, u32) {
    let mut fx = ImageFixture::new(BASE);
    let code = [0x48, 0x8B, 0x81, 0xA8, 0x01, 0x00, 0x00, 0xE8, 0x34, 0x12, 0x00, 0x00, 0x84, 0xC0, 0xC3];
    let start = fx.put_text(&code);
    fx.put_function(start, start + code.len() as u32);
    (fx.image(), start)
}

#[test]
fn draft_wildcards_displacement_and_branch() {
    let (image, start) = image();
    let (text, matches) = PatternDrafter::new(&image).draft(start, 14).unwrap();
    assert_eq!(text, "48 8B 81 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0");
    assert_eq!(matches, 1);
}

#[test]
fn draft_stops_at_function_end() {
    let (image, start) = image();
    let (text, _) = PatternDrafter::new(&image).draft(start, 500).unwrap();
    assert_eq!(text, "48 8B 81 ?? ?? ?? ?? E8 ?? ?? ?? ?? 84 C0 C3");
}

#[test]
fn draft_covers_a_whole_instruction_past_the_requested_length() {
    let (image, start) = image();
    let (text, _) = PatternDrafter::new(&image).draft(start, 2).unwrap();
    assert_eq!(text, "48 8B 81 ?? ?? ?? ??");
}

#[test]
fn draft_from_mid_instruction_names_the_cause() {
    let (image, start) = image();
    let error = PatternDrafter::new(&image).draft(start + 1, 8).unwrap_err();
    assert_eq!(error, StepError::NotInstructionStart(start + 1));
    assert!(error.to_string().contains("not an instruction start"), "{error}");
}

#[test]
fn draft_outside_a_function_names_the_cause() {
    let (image, start) = image();
    let error = PatternDrafter::new(&image).draft(start + 0x100, 8).unwrap_err();
    assert!(error.to_string().contains("not in a function"), "{error}");
}

#[test]
fn draft_reports_more_than_one_match() {
    let mut fx = ImageFixture::new(BASE);
    let code = [0x48, 0x83, 0xEC, 0x28, 0xC3];
    let start = fx.put_text(&code);
    fx.put_text(&code);
    fx.put_function(start, start + 5);
    let (text, matches) = PatternDrafter::new(&fx.image()).draft(start, 5).unwrap();
    assert_eq!(text, "48 83 EC ?? C3");
    assert_eq!(matches, 2);
}

#[test]
fn draft_wildcards_a_rip_relative_displacement() {
    let mut fx = ImageFixture::new(BASE);
    let code = [0x48, 0x8B, 0x05, 0x10, 0x00, 0x00, 0x00, 0xC3];
    let start = fx.put_text(&code);
    fx.put_function(start, start + 8);
    let (text, _) = PatternDrafter::new(&fx.image()).draft(start, 8).unwrap();
    assert_eq!(text, "48 8B 05 ?? ?? ?? ?? C3");
}

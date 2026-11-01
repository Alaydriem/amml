use amml_derive::testing::ImageFixture;
use amml_derive::{FunctionListing, Image, StepError};

const BASE: u64 = 0x1_4000_0000;

// `caller` calls `callee`, then padding; returns the image and the two function starts
fn image() -> (Image, u32, u32) {
    let mut fx = ImageFixture::new(BASE);
    // call rel32 to 0x1010, ret, ten int3
    let mut code = vec![0xE8, 0x0B, 0x00, 0x00, 0x00, 0xC3];
    code.extend([0xCC; 10]);
    let caller = fx.put_text(&code);
    let callee = fx.put_text(&[0xC3]);
    fx.put_function(caller, caller + 6);
    fx.put_function(callee, callee + 1);
    (fx.image(), caller, callee)
}

#[test]
fn listing_marks_function_bounds_and_call_targets() {
    let (image, caller, callee) = image();
    assert_eq!(callee, caller + 0x10);
    let text = FunctionListing::new(&image).listing(caller, None).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], format!("; function {caller:#x}..{:#x}", caller + 6));
    assert!(lines[1].starts_with(&format!("{caller:#x}  e8 0b 00 00 00 ")), "{}", lines[1]);
    assert!(lines[1].ends_with(&format!("call {callee:#x} ; fn")), "{}", lines[1]);
    assert!(lines[2].ends_with("ret"), "{}", lines[2]);
    assert_eq!(lines.len(), 3);
}

#[test]
fn listing_honours_count() {
    let (image, caller, _) = image();
    let text = FunctionListing::new(&image).listing(caller, Some(1)).unwrap();
    assert_eq!(text.lines().count(), 2);
}

#[test]
fn listing_from_padding_names_the_cause() {
    let (image, caller, _) = image();
    let error = FunctionListing::new(&image).listing(caller + 6, None).unwrap_err();
    assert!(error.to_string().contains("not in a function"), "{error}");
}

#[test]
fn listing_from_mid_instruction_names_the_cause() {
    let (image, caller, _) = image();
    let error = FunctionListing::new(&image).listing(caller + 1, None).unwrap_err();
    assert_eq!(error, StepError::NotInstructionStart(caller + 1));
    assert!(error.to_string().contains("not an instruction start"), "{error}");
}

#[test]
fn ranges_list_the_primary_first_and_every_fragment() {
    let mut fx = ImageFixture::new(BASE);
    let fragment = fx.put_text(&[0x90, 0xC3]);
    let primary = fx.put_text(&[0x90, 0xC3]);
    fx.put_function(primary, primary + 2);
    fx.put_fragment(fragment, fragment + 2, primary);
    let text = FunctionListing::new(&fx.image()).ranges(fragment).unwrap();
    assert_eq!(text, format!("primary {primary:#x}..{:#x}\nfragment {fragment:#x}..{:#x}", primary + 2, fragment + 2));
}

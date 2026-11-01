use amml_derive::testing::ImageFixture;
use amml_derive::{FieldKind, FixedFields, Image, StepError};

const BASE: u64 = 0x1_4000_0000;

// mov rax,[rcx+1A8h]; call rel32; ret; returns the image and the function start
fn image() -> (Image, u32) {
    let mut fx = ImageFixture::new(BASE);
    let code = [0x48, 0x8B, 0x81, 0xA8, 0x01, 0x00, 0x00, 0xE8, 0x34, 0x12, 0x00, 0x00, 0xC3];
    let start = fx.put_text(&code);
    fx.put_function(start, start + code.len() as u32);
    (fx.image(), start)
}

#[test]
fn fields_list_every_displacement_and_branch_byte() {
    let (image, start) = image();
    let found = FixedFields::new(&image).fields(start, 12).unwrap();
    let mut expected: Vec<(u32, FieldKind)> = (3..7).map(|at| (start + at, FieldKind::Displacement)).collect();
    expected.extend((8..12).map(|at| (start + at, FieldKind::Branch)));
    assert_eq!(found, expected);
}

#[test]
fn fields_from_mid_instruction_name_the_cause() {
    let (image, start) = image();
    let error = FixedFields::new(&image).fields(start + 1, 4).unwrap_err();
    assert_eq!(error, StepError::NotInstructionStart(start + 1));
}

#[test]
fn fields_outside_a_function_name_the_cause() {
    let (image, start) = image();
    let error = FixedFields::new(&image).fields(start + 0x100, 4).unwrap_err();
    assert_eq!(error, StepError::NotInListedFunction(start + 0x100));
}

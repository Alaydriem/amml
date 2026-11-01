use amml_data::{BuildId, OperandKind};
use amml_derive::testing::ImageFixture;
use amml_derive::{Disasm, FunctionTable, Image, Section, StepError};

const BASE: u64 = 0x1_4000_0000;

// call rel32 from `at` to `target`
fn call(at: u32, target: u32) -> [u8; 5] {
    let rel = target.wrapping_sub(at + 5).to_le_bytes();
    [0xE8, rel[0], rel[1], rel[2], rel[3]]
}

#[test]
fn displacement_of_mov_from_memory() {
    // mov rax, [rcx+0x5A8]
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0x48, 0x8B, 0x81, 0xA8, 0x05, 0x00, 0x00]);
    assert_eq!(Disasm::operand(&fx.image(), at, 1, OperandKind::Displacement).unwrap(), 0x5A8);
}

#[test]
fn scale_factor_counts_the_base_when_it_is_the_index() {
    // lea rax, [rdx+rdx*8]: x * 9; lea rax, [rcx+rdx*4]: the scale alone; mov rax, [rcx]: no index
    let mut fx = ImageFixture::new(BASE);
    let nine = fx.put_text(&[0x48, 0x8D, 0x04, 0xD2]);
    let four = fx.put_text(&[0x48, 0x8D, 0x04, 0x91]);
    let none = fx.put_text(&[0x48, 0x8B, 0x01]);
    let img = fx.image();
    assert_eq!(Disasm::operand(&img, nine, 1, OperandKind::ScaleFactor).unwrap(), 9);
    assert_eq!(Disasm::operand(&img, four, 1, OperandKind::ScaleFactor).unwrap(), 4);
    assert!(Disasm::operand(&img, none, 1, OperandKind::ScaleFactor).is_err());
}

#[test]
fn immediate_of_mov_to_register() {
    // mov eax, 0x1234
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0xB8, 0x34, 0x12, 0x00, 0x00]);
    assert_eq!(Disasm::operand(&fx.image(), at, 1, OperandKind::Immediate).unwrap(), 0x1234);
}

#[test]
fn rip_target_is_returned_as_rva() {
    // lea rcx, [rip+0x100]
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0x48, 0x8D, 0x0D, 0x00, 0x01, 0x00, 0x00]);
    assert_eq!(Disasm::operand(&fx.image(), at, 1, OperandKind::RipTarget).unwrap(), u64::from(at + 7 + 0x100));
}

#[test]
fn branch_target_of_call_is_an_rva() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&call(0x1000, 0x1234));
    assert_eq!(at, 0x1000);
    assert_eq!(Disasm::operand(&fx.image(), at, 0, OperandKind::BranchTarget).unwrap(), 0x1234);
}

#[test]
fn missing_or_mismatched_operand_is_an_error() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0x48, 0x8B, 0x81, 0xA8, 0x05, 0x00, 0x00]);
    let img = fx.image();
    assert!(matches!(
        Disasm::operand(&img, at, 5, OperandKind::Displacement),
        Err(StepError::NoOperand { index: 5, .. })
    ));
    assert!(matches!(
        Disasm::operand(&img, at, 1, OperandKind::BranchTarget),
        Err(StepError::NoOperand { index: 1, .. })
    ));
}

#[test]
fn nth_call_counts_direct_calls_inside_the_function_only() {
    // call A; nop; call B; ret; then outside the function: call C
    let (a, b, c) = (0x2000, 0x3000, 0x4000);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&call(0x1000, a));
    bytes.push(0x90);
    bytes.extend_from_slice(&call(0x1006, b));
    bytes.push(0xC3);
    let end = 0x1000 + bytes.len() as u32;
    bytes.extend_from_slice(&call(end, c));
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&bytes);
    assert_eq!(start, 0x1000);
    fx.put_function(start, end);
    let img = fx.image();
    assert_eq!(Disasm::nth_call(&img, start, 0).unwrap(), a);
    assert_eq!(Disasm::nth_call(&img, start + 6, 1).unwrap(), b);
    assert!(matches!(Disasm::nth_call(&img, start, 2), Err(StepError::CallIndex { nth: 2, found: 2 })));
}

#[test]
fn nth_call_continues_into_chained_fragments() {
    // primary 0x1000: call A; ret. fragment 0x1100: call B; ret
    let (a, b) = (0x2000, 0x3000);
    let mut text = vec![0u8; 0x110];
    text[..5].copy_from_slice(&call(0x1000, a));
    text[5] = 0xC3;
    text[0x100..0x105].copy_from_slice(&call(0x1100, b));
    text[0x105] = 0xC3;
    let functions = FunctionTable::with_fragments(vec![(0x1000, 0x1006)], vec![(0x1100, 0x1106, 0x1000)]);
    let img = Image::new(BASE, vec![Section::new(".text".into(), 0x1000, text, true)], functions, BuildId::new(1, 2));
    assert_eq!(Disasm::nth_call(&img, 0x1000, 1).unwrap(), b);
    assert_eq!(Disasm::nth_call(&img, 0x1100, 0).unwrap(), a);
}

#[test]
fn rva_outside_any_function_is_an_error() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0x90, 0xC3]);
    fx.put_function(start, start + 2);
    let loose = fx.put_text(&[0x90]);
    assert!(matches!(Disasm::nth_call(&fx.image(), loose, 0), Err(StepError::NotInFunction(_))));
}

#[test]
fn displacement_is_rejected_for_rip_relative_and_negative_operands() {
    let mut fx = ImageFixture::new(BASE);
    // lea rcx, [rip+0x100]
    let rip = fx.put_text(&[0x48, 0x8D, 0x0D, 0x00, 0x01, 0x00, 0x00]);
    // mov rax, [rcx-8]
    let neg = fx.put_text(&[0x48, 0x8B, 0x41, 0xF8]);
    let img = fx.image();
    assert!(matches!(Disasm::operand(&img, rip, 1, OperandKind::Displacement), Err(StepError::NoOperand { .. })));
    assert!(matches!(Disasm::operand(&img, neg, 1, OperandKind::Displacement), Err(StepError::NoOperand { .. })));
}

#[test]
fn immediate_on_a_memory_operand_is_an_error() {
    let mut fx = ImageFixture::new(BASE);
    let at = fx.put_text(&[0x48, 0x8B, 0x81, 0xA8, 0x05, 0x00, 0x00]);
    assert!(matches!(Disasm::operand(&fx.image(), at, 1, OperandKind::Immediate), Err(StepError::NoOperand { .. })));
}

#[test]
fn branch_target_of_jmp_is_an_rva() {
    // jmp rel32 from 0x1000 to 0x1800
    let mut fx = ImageFixture::new(BASE);
    let rel = 0x1800u32.wrapping_sub(0x1005).to_le_bytes();
    let at = fx.put_text(&[0xE9, rel[0], rel[1], rel[2], rel[3]]);
    assert_eq!(Disasm::operand(&fx.image(), at, 0, OperandKind::BranchTarget).unwrap(), 0x1800);
}

#[test]
fn indirect_calls_are_not_counted() {
    // call [rip+0x10]; call rax; call B
    let b = 0x3000;
    let mut bytes = vec![0xFF, 0x15, 0x10, 0x00, 0x00, 0x00, 0xFF, 0xD0];
    bytes.extend_from_slice(&call(0x1008, b));
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&bytes);
    fx.put_function(start, start + bytes.len() as u32);
    let img = fx.image();
    assert_eq!(Disasm::nth_call(&img, start, 0).unwrap(), b);
    assert!(matches!(Disasm::nth_call(&img, start, 1), Err(StepError::CallIndex { nth: 1, found: 1 })));
}

#[test]
fn invalid_bytes_inside_a_function_are_an_error_at_their_address() {
    // call A; push es (invalid in 64-bit mode); call B
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&call(0x1000, 0x2000));
    bytes.push(0x06);
    bytes.extend_from_slice(&call(0x1006, 0x3000));
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&bytes);
    fx.put_function(start, start + bytes.len() as u32);
    assert!(matches!(Disasm::nth_call(&fx.image(), start, 1), Err(StepError::BadInstruction(0x1005))));
}

#[test]
fn instruction_truncated_at_the_range_end_is_an_error() {
    // call A; then a call opcode cut off after one byte
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&call(0x1000, 0x2000));
    bytes.extend_from_slice(&[0xE8, 0x00]);
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&bytes);
    fx.put_function(start, start + bytes.len() as u32);
    assert!(matches!(Disasm::nth_call(&fx.image(), start, 1), Err(StepError::BadInstruction(0x1005))));
}

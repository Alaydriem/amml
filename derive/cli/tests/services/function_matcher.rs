use amml_derive::testing::ImageFixture;
use amml_derive::{FunctionMatcher, Image, StepError};

const BASE: u64 = 0x1_4000_0000;

fn rel32(from_next: u32, target: u32) -> [u8; 4] {
    target.wrapping_sub(from_next).to_le_bytes()
}

// One instruction of a test function
enum Op {
    // lea rcx, [rip+string]
    Str(&'static str),
    // call qword ptr [rip+iat slot]
    Import(&'static str),
    // mov eax, imm32
    Mov(u32),
    // add eax, imm8
    Add(i8),
    // movss xmm0, [rip+literal]
    Float(f32),
}

// Puts one function made of `ops` and a ret; returns its start
fn function(fx: &mut ImageFixture, ops: &[Op]) -> u32 {
    // the next text offset is 16-byte aligned, so the start is known before the code is put
    let start = fx.put_text(&[]).next_multiple_of(16);
    let mut code = Vec::new();
    for op in ops {
        let at = start + code.len() as u32;
        match op {
            Op::Str(text) => {
                let mut bytes = text.as_bytes().to_vec();
                bytes.push(0);
                let rva = fx.put_rdata(&bytes);
                code.extend([0x48, 0x8D, 0x0D]);
                code.extend(rel32(at + 7, rva));
            }
            Op::Import(name) => {
                let slot = fx.put_import(name);
                code.extend([0xFF, 0x15]);
                code.extend(rel32(at + 6, slot));
            }
            Op::Add(value) => code.extend([0x83, 0xC0, *value as u8]),
            Op::Float(value) => {
                let rva = fx.put_rdata(&value.to_le_bytes());
                code.extend([0xF3, 0x0F, 0x10, 0x05]);
                code.extend(rel32(at + 8, rva));
            }
            Op::Mov(value) => {
                code.push(0xB8);
                code.extend(value.to_le_bytes());
            }
        }
    }
    code.push(0xC3);
    let placed = fx.put_text(&code);
    fx.put_function(placed, placed + code.len() as u32);
    placed
}

fn candidates(old: &Image, new: &Image, rva: u32) -> Result<Vec<u32>, StepError> {
    let found = FunctionMatcher::new(old, new).candidates(rva, 10)?;
    Ok(found.iter().map(|candidate| candidate.rva).collect())
}

#[test]
fn same_strings_rank_first() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Str("a.b"), Op::Str("c.d")]);

    let mut new = ImageFixture::new(BASE);
    function(&mut new, &[Op::Str("other")]);
    let y = function(&mut new, &[Op::Str("c.d")]);
    let x = function(&mut new, &[Op::Str("a.b"), Op::Str("c.d")]);

    assert_eq!(candidates(&old.image(), &new.image(), wanted).unwrap(), [x, y]);
}

#[test]
fn small_immediates_are_ignored() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Str("only.here"), Op::Mov(8)]);

    let mut new = ImageFixture::new(BASE);
    function(&mut new, &[Op::Mov(8)]);

    assert!(candidates(&old.image(), &new.image(), wanted).unwrap().is_empty());
}

#[test]
fn large_constants_and_imports_match() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Mov(0x12_3456), Op::Import("GetTickCount64")]);

    let mut new = ImageFixture::new(BASE);
    let constant = function(&mut new, &[Op::Mov(0x12_3456)]);
    let both = function(&mut new, &[Op::Mov(0x12_3456), Op::Import("GetTickCount64")]);
    let import = function(&mut new, &[Op::Import("GetTickCount64")]);

    let found = FunctionMatcher::new(&old.image(), &new.image()).candidates(wanted, 10).unwrap();
    let order: Vec<u32> = found.iter().map(|candidate| candidate.rva).collect();
    // equal scores: the function closer in size to the old one comes first
    assert_eq!(order, [both, import, constant]);
    assert_eq!((found[0].shared_constants, found[0].shared_imports, found[0].shared_strings), (1, 1, 0));
}

#[test]
fn no_anchors_is_an_error() {
    let mut old = ImageFixture::new(BASE);
    let bare = function(&mut old, &[Op::Mov(8)]);
    let mut new = ImageFixture::new(BASE);
    function(&mut new, &[Op::Mov(8)]);

    let error = candidates(&old.image(), &new.image(), bare).unwrap_err();
    assert!(error.to_string().contains("nothing to match"), "{error}");
}

#[test]
fn a_negative_immediate_is_an_anchor() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Add(-16)]);
    let mut new = ImageFixture::new(BASE);
    function(&mut new, &[Op::Add(16)]);
    let same = function(&mut new, &[Op::Add(-16)]);

    assert_eq!(candidates(&old.image(), &new.image(), wanted).unwrap(), [same]);
}

#[test]
fn a_float_literal_from_rdata_is_an_anchor() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Float(1.5)]);
    let mut new = ImageFixture::new(BASE);
    function(&mut new, &[Op::Float(2.5)]);
    let same = function(&mut new, &[Op::Float(1.5)]);

    assert_eq!(candidates(&old.image(), &new.image(), wanted).unwrap(), [same]);
}

#[test]
fn top_truncates_the_list() {
    let mut old = ImageFixture::new(BASE);
    let wanted = function(&mut old, &[Op::Str("shared.text")]);
    let mut new = ImageFixture::new(BASE);
    for _ in 0..3 {
        function(&mut new, &[Op::Str("shared.text")]);
    }
    let found = FunctionMatcher::new(&old.image(), &new.image()).candidates(wanted, 2).unwrap();
    assert_eq!(found.len(), 2);
}

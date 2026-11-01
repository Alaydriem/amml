use amml_derive::testing::ImageFixture;
use amml_derive::{CodeRefKind, CodeXref};

const BASE: u64 = 0x1_4000_0000;

fn rel32(from_next: u32, target: u32) -> [u8; 4] {
    target.wrapping_sub(from_next).to_le_bytes()
}

// puts a function made of `code` and registers it; returns its start
fn function(fx: &mut ImageFixture, code: &[u8]) -> u32 {
    let start = fx.put_text(code);
    fx.put_function(start, start + code.len() as u32);
    start
}

fn target(fx: &mut ImageFixture) -> u32 {
    function(fx, &[0xC3])
}

#[test]
fn callers_finds_direct_call_and_tail_jump() {
    let mut fx = ImageFixture::new(BASE);
    let callee = target(&mut fx);
    // the next text offset is 16-byte aligned, so each function start is known before it is put
    let caller = callee + 0x10;
    let mut call = vec![0xE8];
    call.extend(rel32(caller + 5, callee));
    call.push(0xC3);
    assert_eq!(function(&mut fx, &call), caller);
    let tail = caller + 0x10;
    let mut jump = vec![0xE9];
    jump.extend(rel32(tail + 5, callee));
    assert_eq!(function(&mut fx, &jump), tail);

    let image = fx.image();
    let refs = CodeXref::new(&image).callers(callee);
    let summary: Vec<_> = refs.iter().map(|r| (r.site, r.function, r.kind)).collect();
    assert_eq!(summary, [(caller, Some(caller), CodeRefKind::Call), (tail, Some(tail), CodeRefKind::Jump)]);
    assert!(refs[0].text.starts_with("call "), "{}", refs[0].text);
    assert!(refs[1].text.starts_with("jmp "), "{}", refs[1].text);
}

#[test]
fn rip_relative_load_is_found() {
    let mut fx = ImageFixture::new(BASE);
    let value = fx.put_rdata(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let load = fx.put_text(&[]);
    let mut code = vec![0x48, 0x8D, 0x0D];
    code.extend(rel32(load + 7, value));
    code.push(0xC3);
    function(&mut fx, &code);

    let image = fx.image();
    let refs = CodeXref::new(&image).refs_to(value);
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!((refs[0].site, refs[0].kind), (load, CodeRefKind::RipRelative));
    assert!(refs[0].text.starts_with("lea rcx,[") || refs[0].text.starts_with("lea rcx, ["), "{}", refs[0].text);
    assert!(CodeXref::new(&image).callers(value).is_empty());
}

#[test]
fn byte_coincidence_is_not_a_ref() {
    let mut fx = ImageFixture::new(BASE);
    let callee = target(&mut fx);
    let holder = callee + 0x10;
    // mov rax, imm64 whose immediate starts with an E8 rel32 that reaches the callee
    let mut code = vec![0x48, 0xB8, 0xE8];
    code.extend(rel32(holder + 2 + 5, callee));
    code.extend([0x00, 0x00, 0x00, 0xC3]);
    assert_eq!(function(&mut fx, &code), holder);

    let image = fx.image();
    assert!(CodeXref::new(&image).callers(callee).is_empty());
    assert!(CodeXref::new(&image).refs_to(callee).is_empty());
}

#[test]
fn data_pointer_in_rdata_is_found() {
    let mut fx = ImageFixture::new(BASE);
    let callee = target(&mut fx);
    let slot = fx.put_rdata(&(BASE + u64::from(callee)).to_le_bytes());

    let image = fx.image();
    let refs = CodeXref::new(&image).refs_to(callee);
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!((refs[0].site, refs[0].function, refs[0].kind), (slot, None, CodeRefKind::DataPointer));
    assert!(CodeXref::new(&image).callers(callee).is_empty());
}

#[test]
fn rip_relative_operand_before_an_immediate_is_found() {
    let mut fx = ImageFixture::new(BASE);
    let value = fx.put_rdata(&[0; 8]);
    // mov byte [rip+x], imm8
    let byte_store = fx.put_text(&[]);
    let mut code = vec![0xC6, 0x05];
    code.extend(rel32(byte_store + 7, value));
    code.extend([0x01, 0xC3]);
    function(&mut fx, &code);
    // cmp dword [rip+x], imm32
    let compare = fx.put_text(&[]);
    let mut code = vec![0x81, 0x3D];
    code.extend(rel32(compare + 10, value));
    code.extend([0x78, 0x56, 0x34, 0x12, 0xC3]);
    function(&mut fx, &code);

    let image = fx.image();
    let refs = CodeXref::new(&image).refs_to(value);
    let summary: Vec<_> = refs.iter().map(|r| (r.site, r.kind)).collect();
    assert_eq!(summary, [(byte_store, CodeRefKind::RipRelative), (compare, CodeRefKind::RipRelative)]);
}

#[test]
fn near_conditional_jump_is_a_jump() {
    let mut fx = ImageFixture::new(BASE);
    let callee = target(&mut fx);
    let site = fx.put_text(&[]);
    let mut code = vec![0x0F, 0x84];
    code.extend(rel32(site + 6, callee));
    code.push(0xC3);
    function(&mut fx, &code);

    let image = fx.image();
    let refs = CodeXref::new(&image).callers(callee);
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!((refs[0].site, refs[0].kind), (site, CodeRefKind::Jump));
}

#[test]
fn ref_from_a_cold_fragment_names_the_primary_function() {
    let mut fx = ImageFixture::new(BASE);
    let callee = target(&mut fx);
    let primary = function(&mut fx, &[0x90, 0xC3]);
    let fragment = fx.put_text(&[]);
    let mut code = vec![0xE8];
    code.extend(rel32(fragment + 5, callee));
    code.push(0xC3);
    fx.put_text(&code);
    fx.put_fragment(fragment, fragment + 6, primary);

    let image = fx.image();
    let refs = CodeXref::new(&image).callers(callee);
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!((refs[0].site, refs[0].function, refs[0].kind), (fragment, Some(primary), CodeRefKind::Call));
}

#[test]
fn indirect_call_through_rip_is_not_a_caller() {
    let mut fx = ImageFixture::new(BASE);
    let slot = fx.put_rdata(&[0; 8]);
    let site = fx.put_text(&[]);
    // call qword [rip+x]
    let mut code = vec![0xFF, 0x15];
    code.extend(rel32(site + 6, slot));
    code.push(0xC3);
    function(&mut fx, &code);

    let image = fx.image();
    assert!(CodeXref::new(&image).callers(slot).is_empty());
    let refs = CodeXref::new(&image).refs_to(slot);
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!((refs[0].site, refs[0].kind), (site, CodeRefKind::RipRelative));
}

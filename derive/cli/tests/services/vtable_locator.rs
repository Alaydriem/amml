use amml_derive::testing::ImageFixture;
use amml_derive::{StepError, VtableLocator};

const BASE: u64 = 0x1_4000_0000;

fn rel32(from_next: u32, target: u32) -> [u8; 4] {
    target.wrapping_sub(from_next).to_le_bytes()
}

// a one-byte function; returns its start
fn function(fx: &mut ImageFixture) -> u32 {
    fx.put_text(&[0xC3])
}

// code that stores `named` as a vptr through rcx: `lea rax,[rip+named]`, `filler` nops, `mov [rcx],rax`, `ret`;
// the code is a `.pdata` function when `pdata` is set
fn store(fx: &mut ImageFixture, named: u32, filler: usize, pdata: bool) {
    // the next text offset is 16-byte aligned, so the start is known before the code is put
    let site = fx.put_text(&[]);
    let mut code = vec![0x48, 0x8D, 0x05];
    code.extend(rel32(site + 7, named));
    code.extend(std::iter::repeat_n(0x90, filler));
    // `mov [rcx],rax` is 48 89 01
    code.extend([0x48, 0x89, 0x01, 0xC3]);
    let start = fx.put_text(&code);
    if pdata {
        fx.put_function(start, start + code.len() as u32);
    }
}

// tables laid out one after the other behind a non-code qword; returns each table start
fn tables(fx: &mut ImageFixture, tables: &[&[u32]]) -> Vec<u32> {
    let mut bytes = 0u64.to_le_bytes().to_vec();
    let mut starts = Vec::new();
    for slots in tables {
        starts.push(bytes.len() as u32);
        for slot in *slots {
            bytes.extend((BASE + u64::from(*slot)).to_le_bytes());
        }
    }
    let base = fx.put_data(&bytes);
    starts.into_iter().map(|offset| base + offset).collect()
}

// `lea rax,[rip+named]`, `tail`, `ret`, as a `.pdata` function
fn lea_then(fx: &mut ImageFixture, named: u32, tail: &[u8]) {
    let site = fx.put_text(&[]);
    let mut code = vec![0x48, 0x8D, 0x05];
    code.extend(rel32(site + 7, named));
    code.extend(tail);
    code.push(0xC3);
    let start = fx.put_text(&code);
    fx.put_function(start, start + code.len() as u32);
}

// a table of six slots with a stored start, a `lea` of the inner address start+16 followed by `tail`, and the
// result for the function in slot 3
fn inner_lea_case(tail: &[u8]) -> (Result<u32, StepError>, u32, u32) {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..6).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, true);
    lea_then(&mut fx, table + 16, tail);
    (holding(&fx, funcs[3]), funcs[3], table + 24)
}

fn holding(fx: &ImageFixture, function: u32) -> Result<u32, StepError> {
    let image = fx.image();
    VtableLocator::new(&image).table_holding(function)
}

#[test]
fn vtable_holding_finds_the_table() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..8).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, true);
    assert_eq!(holding(&fx, funcs[5]).unwrap(), table);
}

#[test]
fn vtable_holding_refuses_function_in_two_tables() {
    let mut fx = ImageFixture::new(BASE);
    let shared = function(&mut fx);
    let other = function(&mut fx);
    let first = tables(&mut fx, &[&[other, other, shared]])[0];
    let second = tables(&mut fx, &[&[other, other, shared]])[0];
    store(&mut fx, first, 0, true);
    store(&mut fx, second, 0, true);
    assert_eq!(holding(&fx, shared), Err(StepError::InSeveralVtables { rva: shared, count: 2 }));
}

#[test]
fn vtable_holding_refuses_function_in_no_table() {
    let mut fx = ImageFixture::new(BASE);
    let held = function(&mut fx);
    let lonely = function(&mut fx);
    let table = tables(&mut fx, &[&[held, held]])[0];
    store(&mut fx, table, 0, true);
    assert_eq!(holding(&fx, lonely), Err(StepError::NoVtableHolds { rva: lonely }));
}

#[test]
fn vtable_holding_stops_at_adjacent_table_start() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..8).map(|_| function(&mut fx)).collect();
    let starts = tables(&mut fx, &[&funcs[0..4], &funcs[4..8]]);
    store(&mut fx, starts[0], 0, true);
    store(&mut fx, starts[1], 0, true);
    assert_eq!(holding(&fx, funcs[5]).unwrap(), starts[1]);
}

#[test]
fn vtable_holding_refuses_pointer_without_table_start() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..4).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    assert_eq!(holding(&fx, funcs[2]), Err(StepError::UnreadableTable { rva: funcs[2], slot: table + 16 }));
}

#[test]
fn vtable_holding_refuses_when_no_start_is_loaded() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..4).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table + 8, 0, true);
    assert_eq!(holding(&fx, funcs[0]), Err(StepError::UnreadableTable { rva: funcs[0], slot: table }));
}

#[test]
fn vtable_holding_sees_store_in_leaf_without_pdata() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..4).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, false);
    assert_eq!(holding(&fx, funcs[2]).unwrap(), table);
}

#[test]
fn vtable_holding_refuses_a_store_four_instructions_after_the_lea() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..6).map(|_| function(&mut fx)).collect();
    let starts = tables(&mut fx, &[&funcs[0..3], &funcs[3..6]]);
    store(&mut fx, starts[0], 0, true);
    // three nops put the store four instructions after the `lea`
    store(&mut fx, starts[1], 3, true);
    assert_eq!(holding(&fx, funcs[4]), Err(StepError::UnreadableTable { rva: funcs[4], slot: starts[1] + 8 }));
}

#[test]
fn vtable_holding_skips_non_lea_rip_operands() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..6).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, true);
    let site = fx.put_text(&[]);
    let mut load = vec![0x48, 0x8B, 0x05];
    load.extend(rel32(site + 7, table + 16));
    load.push(0xC3);
    let start = fx.put_text(&load);
    fx.put_function(start, start + load.len() as u32);
    assert_eq!(holding(&fx, funcs[3]).unwrap(), table);
}

#[test]
fn vtable_holding_refuses_inner_lea_with_displaced_store() {
    // mov [rcx+8],rax
    let (result, function, slot) = inner_lea_case(&[0x48, 0x89, 0x41, 0x08]);
    assert_eq!(result, Err(StepError::UnreadableTable { rva: function, slot }));
}

#[test]
fn vtable_holding_refuses_inner_lea_with_other_register_store() {
    // mov [rcx],rdx
    let (result, function, slot) = inner_lea_case(&[0x48, 0x89, 0x11]);
    assert_eq!(result, Err(StepError::UnreadableTable { rva: function, slot }));
}

#[test]
fn vtable_holding_refuses_inner_lea_with_indexed_store() {
    // mov [rcx+rdx*8],rax
    let (result, function, slot) = inner_lea_case(&[0x48, 0x89, 0x04, 0xD1]);
    assert_eq!(result, Err(StepError::UnreadableTable { rva: function, slot }));
}

#[test]
fn vtable_holding_refuses_inner_lea_with_reversed_mov() {
    // mov rax,[rcx]
    let (result, function, slot) = inner_lea_case(&[0x48, 0x8B, 0x01]);
    assert_eq!(result, Err(StepError::UnreadableTable { rva: function, slot }));
}

#[test]
fn vtable_holding_refuses_inner_lea_without_a_store() {
    let (result, function, slot) = inner_lea_case(&[]);
    assert_eq!(result, Err(StepError::UnreadableTable { rva: function, slot }));
}

#[test]
fn vtable_holding_refuses_when_one_pointer_is_in_an_unreadable_table() {
    let mut fx = ImageFixture::new(BASE);
    let shared = function(&mut fx);
    let other = function(&mut fx);
    let readable = tables(&mut fx, &[&[other, shared]])[0];
    let unreadable = tables(&mut fx, &[&[other, shared]])[0];
    store(&mut fx, readable, 0, true);
    // the lea of the second table has no store
    lea_then(&mut fx, unreadable, &[]);
    assert_eq!(holding(&fx, shared), Err(StepError::UnreadableTable { rva: shared, slot: unreadable + 8 }));
}

#[test]
fn vtable_holding_counts_one_table_once() {
    let mut fx = ImageFixture::new(BASE);
    let held = function(&mut fx);
    let other = function(&mut fx);
    let table = tables(&mut fx, &[&[other, held, other, held]])[0];
    store(&mut fx, table, 0, true);
    assert_eq!(holding(&fx, held).unwrap(), table);
}

#[test]
fn vtable_holding_makes_a_misaligned_lea_without_store_a_barrier() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..4).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, true);
    // mov rax,imm64 whose immediate holds `lea rax,[rip+table+8]` and a nop; no store follows it
    let site = fx.put_text(&[]);
    let mut code = vec![0x48, 0xB8, 0x48, 0x8D, 0x05];
    // the phantom `lea` is at site + 2 and is 7 bytes long
    code.extend(rel32(site + 9, table + 8));
    code.extend([0x90, 0xC3]);
    fx.put_text(&code);
    assert_eq!(holding(&fx, funcs[2]), Err(StepError::UnreadableTable { rva: funcs[2], slot: table + 16 }));
}

#[test]
fn vtable_holding_counts_a_misaligned_lea_followed_by_a_store() {
    let mut fx = ImageFixture::new(BASE);
    let funcs: Vec<u32> = (0..4).map(|_| function(&mut fx)).collect();
    let table = tables(&mut fx, &[&funcs])[0];
    store(&mut fx, table, 0, true);
    // the immediate of `mov rax,imm64` ends with the `lea`; the next real bytes are `mov [rcx],rax`
    let site = fx.put_text(&[]);
    let mut code = vec![0x48, 0xB8, 0x48, 0x8D, 0x05];
    code.extend(rel32(site + 9, table + 8));
    code.extend([0x90, 0x48, 0x89, 0x01, 0xC3]);
    fx.put_text(&code);
    // known limit: the phantom start is taken
    assert_eq!(holding(&fx, funcs[2]).unwrap(), table + 8);
}

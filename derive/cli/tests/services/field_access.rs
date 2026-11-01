use amml_derive::testing::ImageFixture;
use amml_derive::{AccessKind, FieldAccess, FieldAccessRow};

const BASE: u64 = 0x1_4000_0000;

fn rows_of(code: &[u8]) -> Vec<(AccessKind, u32, u8)> {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(code);
    fx.put_function(start, start + code.len() as u32);
    let rows = match FieldAccess::list(&fx.image(), start) {
        Ok(rows) => rows,
        Err(error) => panic!("{error}"),
    };
    rows.iter().map(|row| (row.kind, row.disp, row.width)).collect()
}

fn row(kind: AccessKind, disp: u32, width: u8) -> FieldAccessRow {
    FieldAccessRow { kind, disp, width, rva: 0 }
}

#[test]
fn fields_lists_reads_writes_and_leas() {
    // mov rbx, rcx; mov eax, [rbx+10h]; mov [rcx+20h], rax; lea rdx, [rcx+30h]; ret
    let code = [0x48, 0x89, 0xCB, 0x8B, 0x43, 0x10, 0x48, 0x89, 0x41, 0x20, 0x48, 0x8D, 0x51, 0x30, 0xC3];
    assert_eq!(
        rows_of(&code),
        vec![(AccessKind::Read, 0x10, 4), (AccessKind::Write, 0x20, 8), (AccessKind::Lea, 0x30, 8)]
    );
}

#[test]
fn fields_stops_tracking_after_overwrite() {
    // mov rcx, rdx; mov eax, [rcx+8]; ret
    let code = [0x48, 0x89, 0xD1, 0x8B, 0x41, 0x08, 0xC3];
    assert_eq!(rows_of(&code), vec![]);
}

#[test]
fn fields_stops_tracking_volatile_registers_after_a_call() {
    // mov eax, [rcx+8]; call $+5; mov eax, [rcx+10h]; ret
    let code = [0x8B, 0x41, 0x08, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x8B, 0x41, 0x10, 0xC3];
    assert_eq!(rows_of(&code), vec![(AccessKind::Read, 0x8, 4)]);
}

#[test]
fn fields_aligns_with_an_inserted_access() {
    let r = |disp| row(AccessKind::Read, disp, 8);
    let w = |disp| row(AccessKind::Write, disp, 8);
    let old = [r(0x8), r(0x10), w(0x18)];
    let new = [r(0x8), r(0x0C), r(0x10), w(0x18)];
    let pairs: Vec<_> = FieldAccess::align(&old, &new)
        .into_iter()
        .map(|(a, b)| (a.map(|row| row.disp), b.map(|row| row.disp)))
        .collect();
    assert_eq!(
        pairs,
        vec![(Some(0x8), Some(0x8)), (None, Some(0x0C)), (Some(0x10), Some(0x10)), (Some(0x18), Some(0x18))]
    );
}

#[test]
fn fields_render_prints_delta_summary() {
    let old = [row(AccessKind::Read, 0x654, 4), row(AccessKind::Read, 0x658, 4), row(AccessKind::Write, 0x660, 4)];
    let new = [row(AccessKind::Read, 0x64C, 4), row(AccessKind::Read, 0x650, 4), row(AccessKind::Write, 0x658, 4)];
    let text = FieldAccess::render(Some(&old), &new);
    assert!(text.contains("0  read  +0x654  +0x64c  -0x8"), "{text}");
    assert!(text.ends_with("deltas: -0x8 x3"), "{text}");
}

#[test]
fn fields_render_one_build() {
    let text = FieldAccess::render(None, &[row(AccessKind::Lea, 0x30, 8)]);
    assert_eq!(text, "0  lea  +0x30  8");
}

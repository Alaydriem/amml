use amml_data::BuildId;
use amml_derive::{FunctionTable, Image, Section};

const LEN: usize = 16;

// two sections that touch: [0x1000, 0x1010) and [0x1010, 0x1020), plus one apart at 0x3000
fn image() -> Image {
    let first: Vec<u8> = (0u8..16).collect();
    let second: Vec<u8> = (16u8..32).collect();
    let apart = vec![0xEE; LEN];
    Image::new(
        0x1_4000_0000,
        vec![
            Section::new(".text".into(), 0x1000, first, true),
            Section::new(".rdata".into(), 0x1010, second, false),
            Section::new(".data".into(), 0x3000, apart, false),
        ],
        FunctionTable::new(Vec::new()),
        BuildId::new(1, 2),
    )
}

#[test]
fn a_read_may_end_exactly_at_the_end_of_its_section() {
    let img = image();
    assert_eq!(img.read(0x1008, 8).unwrap(), &[8, 9, 10, 11, 12, 13, 14, 15]);
    assert_eq!(img.read(0x100F, 1).unwrap(), &[15]);
    assert_eq!(img.read(0x1010, 0).unwrap(), &[] as &[u8]);
}

#[test]
fn a_read_one_byte_past_the_end_of_its_section_fails() {
    let img = image();
    assert!(img.read(0x1009, 8).is_none());
    assert!(img.read(0x100F, 2).is_none());
    assert!(img.read(0x3000, 17).is_none());
}

#[test]
fn a_read_never_continues_into_the_next_section() {
    let img = image();
    assert!(img.read(0x1008, 16).is_none());
    assert_eq!(img.read_u64(0x1010).unwrap(), u64::from_le_bytes([16, 17, 18, 19, 20, 21, 22, 23]));
}

#[test]
fn an_address_outside_every_section_cannot_be_read() {
    let img = image();
    assert!(img.read(0x0FFF, 1).is_none());
    assert!(img.read(0x1020, 1).is_none());
    assert!(img.read(0x2000, 1).is_none());
    assert!(img.read(0x3010, 1).is_none());
}

#[test]
fn a_length_that_overflows_the_address_space_fails_instead_of_wrapping() {
    let img = image();
    assert!(img.read(0x1000, usize::MAX).is_none());
    assert!(img.read(0x1008, usize::MAX - 4).is_none());
}

#[test]
fn integer_reads_at_a_section_edge_need_all_their_bytes() {
    let img = image();
    assert!(img.read_u64(0x1008).is_some());
    assert!(img.read_u64(0x1009).is_none());
    assert!(img.read_u32(0x100C).is_some());
    assert!(img.read_u32(0x100D).is_none());
    assert_eq!(img.read_u32(0x3000).unwrap(), 0xEEEE_EEEE);
    assert!(img.read_u64(0x3009).is_none());
}

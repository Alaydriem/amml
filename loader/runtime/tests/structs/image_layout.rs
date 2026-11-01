use amml_runtime::ImageLayout;
use pelite::pe64::exports::GetProcAddress;
use pelite::pe64::{Pe, PeFile};

use crate::support::Fixtures;

const EXECUTE: u32 = 0x2000_0000;

#[test]
fn the_layout_read_from_headers_agrees_with_pelite_on_every_section() {
    let bytes = std::fs::read(Fixtures::probe_dll()).unwrap();
    let file = PeFile::from_bytes(&bytes).unwrap();

    let layout = ImageLayout::from_headers(&bytes).unwrap();

    assert_eq!(layout.size_of_image(), u64::from(file.optional_header().SizeOfImage));
    let mut sections = 0;
    for section in file.section_headers() {
        sections += 1;
        let start = u64::from(section.VirtualAddress);
        assert_eq!(layout.is_executable(start), section.Characteristics & EXECUTE != 0, "{:?}", section.Name);
    }
    assert!(sections > 1);
    let probe = file.get_export("amml_probe").unwrap().symbol().unwrap();
    assert!(layout.is_executable(u64::from(probe)));
    assert!(!layout.contains(layout.size_of_image()));
}

#[test]
fn bytes_that_are_not_a_complete_pe_give_no_layout() {
    let bytes = std::fs::read(Fixtures::probe_dll()).unwrap();

    assert_eq!(ImageLayout::from_headers(&[0u8; 512]), None);
    assert_eq!(ImageLayout::from_headers(&bytes[..0x100]), None);
    assert_eq!(ImageLayout::from_headers(&[]), None);
}

#[test]
fn an_empty_layout_contains_nothing() {
    let layout = ImageLayout::empty();

    assert!(!layout.contains(0));
    assert!(!layout.is_executable(0));
}

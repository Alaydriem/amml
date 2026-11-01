use amml::EntryPoint;
use pelite::pe64::{Pe, PeFile};

use crate::support::Probe;

#[test]
fn the_entry_point_rva_matches_the_optional_header_of_a_real_image() {
    let bytes = std::fs::read(Probe::dll()).unwrap();
    let expected = PeFile::from_bytes(&bytes).unwrap().optional_header().AddressOfEntryPoint;

    assert_ne!(expected, 0);
    assert_eq!(EntryPoint::rva_from_headers(&bytes), Some(expected));
}

#[test]
fn headers_without_the_dos_magic_have_no_entry_point() {
    let mut bytes = std::fs::read(Probe::dll()).unwrap();
    bytes[0] = b'X';

    assert_eq!(EntryPoint::rva_from_headers(&bytes), None);
}

#[test]
fn headers_cut_before_the_entry_point_field_have_no_entry_point() {
    let bytes = std::fs::read(Probe::dll()).unwrap();
    let nt = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;

    assert_eq!(EntryPoint::rva_from_headers(&bytes[..nt + 24 + 16 + 3]), None);
    assert_eq!(EntryPoint::rva_from_headers(&bytes[..0x20]), None);
}

#[test]
fn a_32_bit_image_has_no_entry_point() {
    let mut bytes = std::fs::read(Probe::dll()).unwrap();
    let nt = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
    bytes[nt + 24..nt + 26].copy_from_slice(&0x10Bu16.to_le_bytes());

    assert_eq!(EntryPoint::rva_from_headers(&bytes), None);
}

#[test]
fn a_process_without_preloader_is_not_the_levilamina_host() {
    assert!(!EntryPoint::is_levilamina_host());
}

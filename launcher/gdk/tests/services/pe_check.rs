use crate::support::PeBytes;
use amml_gdk::PeCheck;

fn check(bytes: &[u8]) -> bool {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.exe");
    std::fs::write(&path, bytes).unwrap();
    PeCheck::is_x64(&path).unwrap()
}

#[test]
fn accepts_x64_header() {
    assert!(check(&PeBytes::x64(4096)));
}

#[test]
fn rejects_x86_truncated_and_zeroed() {
    assert!(!check(&PeBytes::with_machine(4096, 0x014C)));
    assert!(!check(&PeBytes::x64(4096)[..100]));
    assert!(!check(&[0u8; 4096]));
}

#[test]
fn rejects_a_header_offset_outside_the_file() {
    let mut bytes = PeBytes::x64(4096);
    bytes[0x3C..0x40].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(!check(&bytes));
}

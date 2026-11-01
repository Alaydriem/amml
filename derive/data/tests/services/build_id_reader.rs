use std::path::PathBuf;

use amml_data::{BuildId, BuildIdReader, DataError};
use tempfile::TempDir;

struct Fixtures;

impl Fixtures {
    fn path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cli/tests/fixtures/pe").join(name)
    }
}

#[test]
fn the_id_of_a_known_file_is_pinned() {
    let id = BuildIdReader::from_path(&Fixtures::path("small.dll")).unwrap();
    assert_eq!(id, BuildId::new(0x6ae172ec, 0xfe3513fe9d01466a));
    assert_eq!(id.file_stem(), "6ae172ec-fe3513fe9d01466a");
}

#[test]
fn an_added_section_does_not_change_the_id() {
    let plain = BuildIdReader::from_path(&Fixtures::path("small.dll")).unwrap();
    let patched = BuildIdReader::from_path(&Fixtures::path("small-patched.dll")).unwrap();
    assert_eq!(plain, patched);
}

#[test]
fn bytes_and_path_give_the_same_id() {
    let path = Fixtures::path("small.dll");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(BuildIdReader::from_bytes(&bytes).unwrap(), BuildIdReader::from_path(&path).unwrap());
}

#[test]
fn a_file_that_is_not_a_pe_is_rejected_with_its_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("junk.exe");
    std::fs::write(&path, vec![0u8; 4096]).unwrap();
    assert!(matches!(BuildIdReader::from_path(&path), Err(DataError::NotPe { path: p, .. }) if p == path));
}

#[test]
fn a_file_cut_before_the_executable_section_ends_is_rejected() {
    let bytes = std::fs::read(Fixtures::path("small.dll")).unwrap();
    let dir = TempDir::new().unwrap();
    let headers_only = dir.path().join("headers.dll");
    std::fs::write(&headers_only, &bytes[..0x200]).unwrap();
    assert!(matches!(BuildIdReader::from_path(&headers_only), Err(DataError::NotPe { .. })));
}

#[test]
fn a_missing_file_is_an_io_error_naming_the_path() {
    let path = Fixtures::path("absent.dll");
    assert!(matches!(BuildIdReader::from_path(&path), Err(DataError::Io { path: p, .. }) if p == path));
}

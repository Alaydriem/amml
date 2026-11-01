use std::path::PathBuf;

use amml_derive::{DeriveError, ImageLoader};
use pelite::pe64::{Pe, PeFile};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe").join(name)
}

#[test]
fn extra_import_section_does_not_change_build_id() {
    let a = ImageLoader::from_path(&fixture("small.dll")).unwrap();
    let b = ImageLoader::from_path(&fixture("small-patched.dll")).unwrap();
    assert_eq!(b.sections().len(), a.sections().len() + 1);
    assert_eq!(a.build(), b.build());

    let bytes = std::fs::read(fixture("small.dll")).unwrap();
    let header_stamp = PeFile::from_bytes(&bytes).unwrap().file_header().TimeDateStamp;
    assert_eq!(a.build().timestamp, header_stamp);
    assert_ne!(a.build().text_hash, 0);
}

#[test]
fn function_table_comes_from_pdata() {
    let bytes = std::fs::read(fixture("small.dll")).unwrap();
    let file = PeFile::from_bytes(&bytes).unwrap();
    let probe = file.exports().unwrap().by().unwrap().name("amml_probe").unwrap().symbol().unwrap();

    let img = ImageLoader::from_pe_bytes(&bytes).unwrap();
    let (start, end) = img.functions().containing(probe).unwrap();
    assert!(start < end && img.functions().is_start(start));
    assert!(img.section_of(start).unwrap().executable);
    assert!(img.read(start, 4).is_some());
}

#[test]
fn non_pe_input_is_rejected() {
    assert!(matches!(ImageLoader::from_pe_bytes(b"not a pe"), Err(DeriveError::NotPe(_))));
}

#[test]
fn missing_file_names_the_path() {
    let path = fixture("absent.dll");
    assert!(matches!(ImageLoader::from_path(&path), Err(DeriveError::Io { path: p, .. }) if p == path));
}

#[test]
fn image_without_an_executable_section_is_rejected() {
    let mut bytes = std::fs::read(fixture("small.dll")).unwrap();
    let name = bytes.windows(8).position(|w| w == b".text\0\0\0").unwrap();
    // Characteristics is the last u32 of the 40-byte section header
    let at = name + 36;
    let mut flags = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    flags &= !0x2000_0000;
    bytes[at..at + 4].copy_from_slice(&flags.to_le_bytes());
    assert!(matches!(ImageLoader::from_pe_bytes(&bytes), Err(DeriveError::NotPe(_))));
}

#[test]
fn the_loader_takes_its_build_id_from_the_shared_reader() {
    for name in ["small.dll", "small-patched.dll"] {
        let image = ImageLoader::from_path(&fixture(name)).unwrap();
        assert_eq!(image.build(), amml_data::BuildIdReader::from_path(&fixture(name)).unwrap(), "{name}");
    }
    let image = ImageLoader::from_path(&fixture("small.dll")).unwrap();
    assert_eq!(image.build().file_stem(), "6ae172ec-fe3513fe9d01466a");
}

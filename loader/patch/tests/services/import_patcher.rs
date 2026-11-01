use amml_data::BuildIdReader;
use amml_patch::{ImportPatcher, PatchError};
use pelite::pe64::{Pe, PeFile};

use crate::support::{ImportList, PeFixture};

mod edge_cases;
mod load_library;

#[test]
fn the_patched_file_keeps_every_original_import_and_lists_the_new_one() {
    let input = PeFixture::small();
    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    let before = ImportList::read(&input);
    let after = ImportList::read(&output);
    assert!(!before.is_empty());
    assert_eq!(&after[..before.len()], &before[..]);
    assert_eq!(&after[before.len()..], ["amml.dll!amml_anchor"]);
}

#[test]
fn the_new_section_is_aligned_writable_and_covered_by_the_image_size() {
    let input = PeFixture::small();
    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    let before = PeFile::from_bytes(&input).unwrap();
    let after = PeFile::from_bytes(&output).unwrap();
    let sections = after.section_headers();
    assert_eq!(sections.iter().count(), before.section_headers().iter().count() + 1);
    let added = sections.iter().last().unwrap();
    assert_eq!(added.name_bytes(), b".amml");
    assert_eq!(added.Characteristics, 0xC000_0040);
    let optional = after.optional_header();
    assert_eq!(added.VirtualAddress % optional.SectionAlignment, 0);
    assert_eq!(added.PointerToRawData % optional.FileAlignment, 0);
    assert_eq!(added.SizeOfRawData % optional.FileAlignment, 0);
    assert_eq!(output.len(), (added.PointerToRawData + added.SizeOfRawData) as usize);
    assert!(optional.SizeOfImage >= added.VirtualAddress + added.VirtualSize);
    assert_eq!(optional.SizeOfImage % optional.SectionAlignment, 0);
    assert_eq!(after.data_directory()[1].VirtualAddress, added.VirtualAddress);
    assert_eq!(after.data_directory()[11].VirtualAddress, 0);
    assert_eq!(after.data_directory()[11].Size, 0);
}

#[test]
fn the_build_id_survives_the_patch() {
    let input = PeFixture::small();
    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    assert_eq!(BuildIdReader::from_bytes(&input).unwrap(), BuildIdReader::from_bytes(&output).unwrap());
}

#[test]
fn patching_twice_is_refused() {
    let once = ImportPatcher::add_import(&PeFixture::small(), "amml.dll", "amml_anchor").unwrap();

    let error = ImportPatcher::add_import(&once, "amml.dll", "amml_anchor").unwrap_err();
    assert!(matches!(error, PatchError::AlreadyPatched(ref dll) if dll == "amml.dll"), "{error}");
    let error = ImportPatcher::add_import(&once, "AMML.DLL", "other").unwrap_err();
    assert!(matches!(error, PatchError::AlreadyPatched(_)), "{error}");
}

#[test]
fn a_full_section_table_is_reported_and_no_section_is_moved() {
    let mut current = PeFixture::small();
    let mut patched = 0;
    loop {
        let name = format!("extra{patched}.dll");
        match ImportPatcher::add_import(&current, &name, "symbol") {
            Ok(next) => current = next,
            Err(PatchError::NoHeaderRoom) => break,
            Err(error) => panic!("{error}"),
        }
        patched += 1;
        assert!(patched < 64, "the header never filled up");
    }
    assert!(patched >= 1);
    let imports = ImportList::read(&current);
    assert!(imports.contains(&format!("extra{}.dll!symbol", patched - 1)));
}

#[test]
fn input_that_is_not_a_pe_file_is_refused() {
    let error = ImportPatcher::add_import(b"not an executable", "amml.dll", "amml_anchor").unwrap_err();
    assert!(matches!(error, PatchError::Malformed(_)), "{error}");
    let mut truncated = PeFixture::small();
    truncated.truncate(0x200);
    assert!(ImportPatcher::add_import(&truncated, "amml.dll", "amml_anchor").is_err());
}

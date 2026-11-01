use amml_patch::ImportPatcher;
use pelite::pe64::{Pe, PeFile};

use super::load_library::StubLoad;
use crate::support::{ImportList, PeFixture};

/// Reads and writes the data directory entries of a PE file.
struct Directory;

impl Directory {
    fn entry_offset(bytes: &[u8], index: usize) -> usize {
        let pe = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
        pe + 24 + 112 + index * 8
    }

    fn read(bytes: &[u8], index: usize) -> (u32, u32) {
        let at = Self::entry_offset(bytes, index);
        (
            u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()),
            u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()),
        )
    }

    fn write(bytes: &mut [u8], index: usize, address: u32, size: u32) {
        let at = Self::entry_offset(bytes, index);
        bytes[at..at + 4].copy_from_slice(&address.to_le_bytes());
        bytes[at + 4..at + 8].copy_from_slice(&size.to_le_bytes());
    }
}

/// A word-by-word reference checksum, written independently of `PeChecksum`.
struct ReferenceChecksum;

impl ReferenceChecksum {
    fn of(bytes: &[u8]) -> u32 {
        let field = Directory::entry_offset(bytes, 0) - 112 + 64;
        let mut copy = bytes.to_vec();
        copy[field..field + 4].fill(0);
        if copy.len() % 2 == 1 {
            copy.push(0);
        }
        let mut sum: u64 = 0;
        for index in (0..copy.len()).step_by(2) {
            sum += u64::from(u16::from_le_bytes([copy[index], copy[index + 1]]));
        }
        while sum >> 16 != 0 {
            sum = (sum & 0xffff) + (sum >> 16);
        }
        (sum + bytes.len() as u64) as u32
    }
}

fn padded_small() -> Vec<u8> {
    let mut bytes = PeFixture::small();
    bytes.resize(bytes.len().next_multiple_of(8), 0);
    bytes
}

#[test]
fn an_appended_overlay_is_kept_and_the_file_still_parses_and_loads() {
    let mut input = padded_small();
    input.extend_from_slice(b"overlay data that is not a certificate");
    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    PeFile::from_bytes(&output).unwrap();
    let imports = ImportList::read(&output);
    let before = ImportList::read(&input);
    assert_eq!(&imports[..before.len()], &before[..]);
    assert_eq!(&imports[before.len()..], ["amml.dll!amml_anchor"]);
    assert!(output.windows(7).any(|window| window == b"overlay"));
    StubLoad::assert_loads_with_stub(&output);
}

#[test]
fn a_trailing_certificate_is_cut_off_and_its_directory_entry_cleared() {
    let mut input = padded_small();
    let start = input.len() as u32;
    input.extend_from_slice(&[0xCE; 64]);
    Directory::write(&mut input, 4, start, 64);

    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    assert_eq!(Directory::read(&output, 4), (0, 0));
    assert!(!output.windows(16).any(|window| window == [0xCE; 16]));
    let mut unsigned = padded_small();
    Directory::write(&mut unsigned, 4, 0, 0);
    assert_eq!(output, ImportPatcher::add_import(&unsigned, "amml.dll", "amml_anchor").unwrap());
}

#[test]
fn a_certificate_followed_by_other_data_is_not_cut_but_its_entry_is_cleared() {
    let mut input = padded_small();
    let start = input.len() as u32;
    input.extend_from_slice(&[0xCE; 64]);
    input.extend_from_slice(b"tail");
    Directory::write(&mut input, 4, start, 64);

    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    assert_eq!(Directory::read(&output, 4), (0, 0));
    assert!(output.windows(16).any(|window| window == [0xCE; 16]));
}

#[test]
fn a_bound_import_entry_is_cleared() {
    let mut input = PeFixture::small();
    Directory::write(&mut input, 11, 0x100, 0x40);

    let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();

    assert_eq!(Directory::read(&output, 11), (0, 0));
}

#[test]
fn the_stored_checksum_matches_an_independent_computation() {
    for input in [PeFixture::small(), padded_small()] {
        let output = ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").unwrap();
        let stored = PeFile::from_bytes(&output).unwrap().optional_header().CheckSum;
        assert_eq!(stored, ReferenceChecksum::of(&output));
    }
}

#[test]
fn a_directory_table_that_runs_past_the_optional_header_is_refused() {
    let mut input = PeFixture::small();
    let pe = u32::from_le_bytes(input[0x3c..0x40].try_into().unwrap()) as usize;
    input[pe + 24 + 108..pe + 24 + 112].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
    assert!(matches!(
        ImportPatcher::add_import(&input, "amml.dll", "amml_anchor"),
        Err(amml_patch::PatchError::Malformed(_))
    ));
    input.truncate(pe + 24 + 20);
    assert!(ImportPatcher::add_import(&input, "amml.dll", "amml_anchor").is_err());
}

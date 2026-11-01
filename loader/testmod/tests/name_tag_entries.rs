use std::collections::HashMap;

use amml_game::{EntryResolver, GameEntries};
use amml_sdk::SdkError;
use amml_testmod::NameTagEntries;

struct Known(HashMap<&'static str, u64>);

impl EntryResolver for Known {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.0.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }
}

fn entries(vtable: u64, slot: u64) -> GameEntries {
    let known = Known(HashMap::from([
        (NameTagEntries::VTABLE.as_str(), vtable),
        (NameTagEntries::EXTRACT.as_str(), slot),
        (NameTagEntries::SIZE.as_str(), 144),
    ]));
    GameEntries::resolve(&known, &NameTagEntries::wanted()).unwrap()
}

#[test]
fn a_null_vtable_gives_no_target_and_a_live_one_gives_its_slot() {
    assert_eq!(NameTagEntries::target(&entries(0, 3)), None);

    let table = [0_usize, 0, 0, 0xC0DE];
    assert_eq!(NameTagEntries::target(&entries(table.as_ptr() as u64, 3)), Some(0xC0DE));
}

#[test]
fn a_null_slot_gives_no_target() {
    let table = [0_usize; 4];

    assert_eq!(NameTagEntries::target(&entries(table.as_ptr() as u64, 3)), None);
}

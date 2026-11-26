use amml::{HashedStringLayout30, HashedStringLayout38, NameReader, PlayerActionDetours, StringViewLayout};

// A 38h-layout HashedString built in memory: the hash, then the variant storage, the index byte, then the last match.
#[repr(C, align(8))]
struct Hashed38([u8; 0x38]);

impl Hashed38 {
    fn with(index: u8, storage: [u64; 4]) -> Self {
        let mut bytes = [0u8; 0x38];
        for (n, word) in storage.iter().enumerate() {
            bytes[8 + n * 8..16 + n * 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes[0x28] = index;
        Self(bytes)
    }

    fn address(&self) -> usize {
        std::ptr::from_ref(self).expose_provenance()
    }
}

// A std::string storage of 20h bytes: a short text in place (capacity 15), or a pointer to a longer one.
fn string_storage(text: &'static [u8]) -> [u64; 4] {
    if text.len() < 16 {
        let mut inline = [0u8; 16];
        inline[..text.len()].copy_from_slice(text);
        let low = u64::from_le_bytes(inline[..8].try_into().unwrap());
        let high = u64::from_le_bytes(inline[8..].try_into().unwrap());
        [low, high, text.len() as u64, 15]
    } else {
        [text.as_ptr().expose_provenance() as u64, 0, text.len() as u64, text.len() as u64]
    }
}

#[test]
fn only_the_two_known_layouts_get_a_reader() {
    assert!(NameReader::new(0).is_some());
    assert!(NameReader::new(1).is_some());
    assert!(NameReader::new(2).is_none());
}

#[test]
fn the_38h_layout_reads_a_short_string_a_long_string_and_a_view() {
    let reader = NameReader::new(NameReader::LAYOUT_38).unwrap();
    let short = Hashed38::with(0, string_storage(b"minecraft:dirt"));
    let long_text: &'static [u8] = b"minecraft:netherite_pickaxe_long";
    let long = Hashed38::with(0, string_storage(long_text));
    let view_text: &'static [u8] = b"minecraft:iron_pickaxe";
    let view = Hashed38::with(1, [view_text.as_ptr().expose_provenance() as u64, view_text.len() as u64, 0, 0]);

    // SAFETY: each value is a live buffer of the 38h layout; the texts are statics.
    unsafe {
        assert_eq!(reader.read(short.address()).as_deref(), Some("minecraft:dirt"));
        assert_eq!(reader.read(long.address()).as_deref(), Some("minecraft:netherite_pickaxe_long"));
        assert_eq!(reader.read(view.address()).as_deref(), Some("minecraft:iron_pickaxe"));
    }
}

#[test]
fn an_unknown_variant_index_and_a_null_address_read_nothing() {
    let reader = NameReader::new(NameReader::LAYOUT_38).unwrap();
    let bad = Hashed38::with(2, string_storage(b"minecraft:dirt"));
    // SAFETY: a live buffer of the 38h layout, and a null address the reader refuses before any read.
    unsafe {
        assert_eq!(reader.read(bad.address()), None);
        assert_eq!(reader.read(0), None);
    }
}

#[test]
fn the_30h_layout_reads_the_string_at_8() {
    let reader = NameReader::new(NameReader::LAYOUT_30).unwrap();
    // The 30h layout is the 38h buffer's first 30h bytes: the hash, then the std::string.
    let name = Hashed38::with(0, string_storage(b"minecraft:stone"));
    // SAFETY: the buffer holds a live 30h-layout HashedString in its first 30h bytes.
    unsafe { assert_eq!(reader.read(name.address()).as_deref(), Some("minecraft:stone")) };
    assert_eq!(size_of::<HashedStringLayout30>(), 0x30);
    assert_eq!(size_of::<HashedStringLayout38>(), 0x38);
    assert_eq!(size_of::<StringViewLayout>(), 0x10);
}

#[test]
fn a_refused_use_writes_the_games_own_refusal() {
    let mut interaction = [0xAAu8; 8];
    // SAFETY: the buffer has 8 writable bytes; null is refused without a write.
    unsafe {
        PlayerActionDetours::write_refused_interaction(interaction.as_mut_ptr());
        PlayerActionDetours::write_refused_interaction(std::ptr::null_mut());
    }
    assert_eq!(interaction, [3, 0, 0, 0, 0, 0xAA, 0xAA, 0xAA]);
    let mut use_on = [0xAAu8; 2];
    // SAFETY: the buffer has 2 writable bytes.
    unsafe { PlayerActionDetours::write_refused_use_on(use_on.as_mut_ptr()) };
    assert_eq!(use_on, [0, 0xAA]);
}

#[test]
fn the_services_table_has_an_inventory_table() {
    let harness = crate::support::Harness::start();
    // SAFETY: the loader's tables live for the process.
    let services = unsafe { &*amml_abi::AmmlApi::services(harness.state.api()).unwrap() };
    assert!(services.size >= amml_abi::AmmlServices::SIZE_V5);
    // SAFETY: a table of v5 or later points to the loader's static inventory table.
    let inventory = unsafe { &*services.inventory };
    assert!(inventory.size >= amml_abi::AmmlInventoryTable::SIZE_V1);
}

use amml_data::EntryKind;
use amml_runtime::{LeviLaminaSource, SourceNote};

use crate::support::{BASE, DATA_START, Fixtures, IMAGE_SIZE};

fn lookup(name: &str) -> Option<u64> {
    match name {
        "fn_sym" => Some(BASE + 0x1234),
        "global_sym" => Some(BASE + 0x8),
        "slot_sym" => Some(BASE + 0x40),
        "below_base_sym" => Some(BASE - 1),
        "past_image_sym" => Some(BASE + IMAGE_SIZE),
        "far_sym" => Some(0x7ffa_0000_1000),
        "data_fn_sym" => Some(BASE + DATA_START + 0x10),
        "data_global_sym" => Some(BASE + DATA_START + 0x10),
        "heap_global_sym" => Some(0x174_8127_5040),
        _ => None,
    }
}

#[test]
fn levilamina_source_converts_addresses_to_rvas_and_skips_entries_without_msvc_name() {
    let set = Fixtures::set(vec![
        Fixtures::entry("Fn", EntryKind::Function, Some("fn_sym")),
        Fixtures::entry("Global", EntryKind::Global, Some("global_sym")),
        Fixtures::entry("NoName", EntryKind::Function, None),
        Fixtures::entry("Slot", EntryKind::VSlot, Some("slot_sym")),
        Fixtures::entry("Unknown", EntryKind::Function, Some("not_found")),
    ]);
    let source = LeviLaminaSource::new(lookup, set, BASE, Fixtures::layout());
    let wanted = ["Fn", "Global", "NoName", "Slot", "Unknown", "NotInSet"].map(Fixtures::id);
    let outcome = source.resolve_all(&wanted);
    let values: Vec<_> =
        outcome.found.iter().map(|(id, resolved)| (id.0.as_str(), resolved.kind, resolved.value)).collect();
    assert_eq!(values, vec![("Fn", EntryKind::Function, 0x1234), ("Global", EntryKind::Global, 0x8)]);
    assert!(outcome.found.values().all(|resolved| resolved.recipes_agreeing == 0));
    assert_eq!(outcome.notes, vec![SourceNote::LookupMissing(Fixtures::id("Unknown"))]);
}

#[test]
fn an_address_outside_the_image_is_dropped_with_a_note_for_every_kind() {
    let set = Fixtures::set(vec![
        Fixtures::entry("Below", EntryKind::Function, Some("below_base_sym")),
        Fixtures::entry("Past", EntryKind::Function, Some("past_image_sym")),
        Fixtures::entry("Far", EntryKind::Function, Some("far_sym")),
        Fixtures::entry("Heap", EntryKind::Global, Some("heap_global_sym")),
    ]);
    let source = LeviLaminaSource::new(lookup, set, BASE, Fixtures::layout());
    let wanted = ["Below", "Past", "Far", "Heap"].map(Fixtures::id);

    let outcome = source.resolve_all(&wanted);

    assert!(outcome.found.is_empty());
    let note = |name: &str, address: u64| SourceNote::LookupOutsideImage {
        id: Fixtures::id(name),
        address,
        module_base: BASE,
        size_of_image: IMAGE_SIZE,
    };
    assert_eq!(
        outcome.notes,
        vec![
            note("Below", BASE - 1),
            note("Past", BASE + IMAGE_SIZE),
            note("Far", 0x7ffa_0000_1000),
            note("Heap", 0x174_8127_5040)
        ]
    );
}

#[test]
fn a_function_outside_an_executable_section_is_dropped_but_a_global_in_data_is_kept() {
    let set = Fixtures::set(vec![
        Fixtures::entry("DataFn", EntryKind::Function, Some("data_fn_sym")),
        Fixtures::entry("DataGlobal", EntryKind::Global, Some("data_global_sym")),
    ]);
    let source = LeviLaminaSource::new(lookup, set, BASE, Fixtures::layout());

    let outcome = source.resolve_all(&["DataFn", "DataGlobal"].map(Fixtures::id));

    let values: Vec<_> = outcome.found.iter().map(|(id, resolved)| (id.0.as_str(), resolved.value)).collect();
    assert_eq!(values, vec![("DataGlobal", DATA_START + 0x10)]);
    assert_eq!(
        outcome.notes,
        vec![SourceNote::LookupNotExecutable { id: Fixtures::id("DataFn"), rva: DATA_START + 0x10 }]
    );
}

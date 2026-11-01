use amml_msvc::MsvcMap;

use crate::support::FakeMap;

const KEYS: [(&str, u64); 6] =
    [("Count", 1), ("Damage", 2), ("Items", 3), ("Name", 4), ("Slot", 5), ("a-key-longer-than-the-small-string", 6)];

#[test]
fn finds_every_key_of_a_tree_and_its_value() {
    let fake = FakeMap::build(&KEYS);

    for (key, value) in KEYS {
        // SAFETY: the fake is a live tree laid out as the MSVC STL lays it out.
        assert_eq!(unsafe { fake.map().find(key) }, Some(&value), "{key}");
    }
    assert_eq!(fake.map().len(), 6);
}

#[test]
fn a_key_that_is_not_in_the_tree_gives_none_before_between_and_after_the_keys() {
    let fake = FakeMap::build(&KEYS);

    for key in ["", "Aaa", "Damag", "Damages", "Item", "Itemss", "Zzz", "a-key"] {
        // SAFETY: the fake is a live tree.
        assert_eq!(unsafe { fake.map().find(key) }, None, "{key}");
    }
}

#[test]
fn an_empty_map_and_a_null_head_give_none() {
    let fake = FakeMap::build(&[]);
    // SAFETY: the fake is a live empty tree.
    assert_eq!(unsafe { fake.map().find("Items") }, None);
    assert!(fake.map().is_empty());

    let null = [0u64; 2];
    // SAFETY: a null head is a map the walk must refuse before it reads.
    let map = unsafe { &*null.as_ptr().cast::<MsvcMap<u64>>() };
    // SAFETY: as above.
    assert_eq!(unsafe { map.find("Items") }, None);
}

#[test]
fn a_tree_with_a_cycle_ends_the_walk_with_none() {
    let mut fake = FakeMap::build(&KEYS);
    fake.make_cyclic();

    // SAFETY: the nodes are live; only the child pointer loops.
    assert_eq!(unsafe { fake.map().find("Aaa") }, None);
}

#[test]
fn for_each_visits_every_element_in_key_order() {
    let fake = FakeMap::build(&[("Slot", 5), ("Count", 1), ("Name", 4), ("Items", 3), ("Damage", 2)]);
    let mut seen = Vec::new();

    // SAFETY: the fake is a live tree.
    let complete = unsafe { fake.map().for_each(|key, value| seen.push((key.to_vec(), *value))) };

    assert!(complete);
    let expected: Vec<(Vec<u8>, u64)> = [("Count", 1), ("Damage", 2), ("Items", 3), ("Name", 4), ("Slot", 5)]
        .iter()
        .map(|(key, value)| (key.as_bytes().to_vec(), *value))
        .collect();
    assert_eq!(seen, expected);
}

#[test]
fn for_each_on_an_empty_map_visits_nothing_and_on_a_null_head_fails() {
    let fake = FakeMap::build(&[]);
    let mut visits = 0;
    // SAFETY: the fake is a live empty tree.
    assert!(unsafe { fake.map().for_each(|_, _| visits += 1) });
    assert_eq!(visits, 0);

    let null = [0u64; 2];
    // SAFETY: a null head is a map the walk must refuse before it reads.
    let map = unsafe { &*null.as_ptr().cast::<MsvcMap<u64>>() };
    // SAFETY: as above.
    assert!(!unsafe { map.for_each(|_, _| visits += 1) });
    assert_eq!(visits, 0);
}

#[test]
fn for_each_on_a_tree_with_a_cycle_stops_and_fails() {
    let mut fake = FakeMap::build(&KEYS);
    fake.make_cyclic();
    let mut visits = 0;

    // SAFETY: the nodes are live; only the child pointer loops.
    let complete = unsafe { fake.map().for_each(|_, _| visits += 1) };

    assert!(!complete);
    assert!(visits <= KEYS.len());
}

#[test]
fn the_map_is_the_head_pointer_and_the_size() {
    assert_eq!(size_of::<MsvcMap<u64>>(), 16);
}

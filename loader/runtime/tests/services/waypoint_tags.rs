use std::collections::HashMap;

use amml_runtime::{SavedWaypoint, WaypointTags, WaypointWrite};

// The 20 bytes of an `optional<mce::Color>`: four floats, the engaged byte and three bytes of padding.
fn color_bytes(lanes: [f32; 4], engaged: u8, padding: u8) -> [u8; 20] {
    let mut bytes = [padding; 20];
    for (index, lane) in lanes.iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&lane.to_le_bytes());
    }
    bytes[16] = engaged;
    bytes
}

// The memory of the waypoints, as the loader writes to it: the tag goes in as four floats and the engaged byte.
struct Waypoints {
    colors: HashMap<usize, [u8; 20]>,
}

impl Waypoints {
    fn write(&mut self, waypoint: usize, write: WaypointWrite) {
        let bytes = self.colors.get_mut(&waypoint).unwrap();
        match write {
            WaypointWrite::Tag(tag) => {
                for (index, lane) in tag.iter().enumerate() {
                    bytes[index * 4..index * 4 + 4].copy_from_slice(&lane.to_le_bytes());
                }
                bytes[16] = 1;
            }
            WaypointWrite::Saved(saved) => *bytes = saved,
        }
    }

    fn packed(&self, waypoint: usize) -> u32 {
        let bytes = self.colors[&waypoint];
        let lane = |index: usize| {
            f32::from_le_bytes([bytes[index * 4], bytes[index * 4 + 1], bytes[index * 4 + 2], bytes[index * 4 + 3]])
        };
        WaypointTags::pack([lane(0), lane(1), lane(2), lane(3)])
    }
}

#[test]
fn tag_round_trips_the_index() {
    for index in [0u16, 1, 2, 127, 128, 255, 256, 257, 1000, 4095, 65535] {
        let packed = WaypointTags::pack(WaypointTags::encode(index));
        assert_eq!(WaypointTags::decode(packed), Some(index), "index {index}");
    }
    // Every colour of the game's own packing that is not a tag gives `None`.
    assert_eq!(WaypointTags::decode(0xFFFF_FFFF), None);
    assert_eq!(WaypointTags::decode(0xFF00_0000), None);
    assert_eq!(WaypointTags::decode(0x80A5_0001), None);
    assert_eq!(WaypointTags::decode(WaypointTags::pack([1.0, 0.0, 0.0, 1.0])), None);
}

#[test]
fn colours_are_restored_after_render() {
    let originals = [
        (0x1000usize, color_bytes([0.2, 0.4, 0.6, 1.0], 1, 0x11)),
        (0x2000, color_bytes([1.0, 0.0, 0.5, 0.25], 1, 0x22)),
        // An empty optional: the vanilla blit draws white, and the bytes must come back as they were.
        (0x3000, color_bytes([0.0, 0.0, 0.0, 0.0], 0, 0x33)),
    ];
    let mut memory = Waypoints { colors: originals.iter().copied().collect() };
    let mut tags = WaypointTags::new();
    let found = originals
        .iter()
        .enumerate()
        .map(|(index, (at, bytes))| SavedWaypoint::new(*at, *bytes, (index != 1).then_some(100 + index as i64)));

    assert!(tags.tag(found, |waypoint, write| memory.write(waypoint, write)));

    assert!(tags.is_active());
    for (index, (at, bytes)) in originals.iter().enumerate() {
        assert_ne!(memory.colors[at], *bytes);
        let saved = tags.lookup(memory.packed(*at)).unwrap();
        assert_eq!(saved.waypoint(), *at);
        assert_eq!(saved.actor(), (index != 1).then_some(100 + index as i64));
    }
    assert_eq!(tags.lookup(memory.packed(0x3000)).unwrap().vanilla_packed(), 0xFFFF_FFFF);
    assert_eq!(tags.lookup(memory.packed(0x2000)).unwrap().vanilla_packed(), WaypointTags::pack([1.0, 0.0, 0.5, 0.25]));

    tags.restore(|waypoint, bytes| {
        memory.colors.insert(waypoint, *bytes);
    });

    assert!(!tags.is_active());
    for (at, bytes) in &originals {
        assert_eq!(memory.colors[at], *bytes, "waypoint {at:#x}");
    }
    assert!(tags.lookup(WaypointTags::pack(WaypointTags::encode(0))).is_none());
}

#[test]
fn a_frame_with_too_many_waypoints_tags_nothing() {
    let found = (0..=WaypointTags::CAPACITY).map(|index| SavedWaypoint::new(0x1000 + index * 0x100, [0; 20], None));
    let mut tags = WaypointTags::new();
    let mut writes = 0;

    assert!(!tags.tag(found, |_, _| writes += 1));

    assert_eq!(writes, 0);
    assert!(!tags.is_active());
}

#[test]
fn restore_without_a_frame_writes_nothing() {
    let mut tags = WaypointTags::new();
    let mut writes = 0;

    tags.restore(|_, _| writes += 1);

    assert_eq!(writes, 0);
}

#[test]
fn a_new_frame_restores_the_open_one_first() {
    let first = (0x1000usize, color_bytes([0.2, 0.4, 0.6, 1.0], 1, 0x11));
    let second = (0x2000usize, color_bytes([1.0, 0.0, 0.5, 0.25], 0, 0x22));
    let mut memory = Waypoints { colors: [first, second].into_iter().collect() };
    let mut tags = WaypointTags::new();
    assert!(tags.tag([SavedWaypoint::new(first.0, first.1, None)], |at, write| memory.write(at, write)));
    assert_ne!(memory.colors[&first.0], first.1);

    // A second frame starts while the first one is open, and holds another waypoint.
    assert!(tags.tag([SavedWaypoint::new(second.0, second.1, None)], |at, write| memory.write(at, write)));

    assert_eq!(memory.colors[&first.0], first.1);
    assert_ne!(memory.colors[&second.0], second.1);
    assert_eq!(tags.lookup(memory.packed(second.0)).unwrap().waypoint(), second.0);
}

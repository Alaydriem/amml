use amml_runtime::EmitterEntry;

#[test]
fn local_position_in_the_window() {
    assert_eq!(EmitterEntry::local([2, 64, 15], [0, 4, 0]), Some([18, 16, 31]));
    assert_eq!(EmitterEntry::local([-16, 48, -16], [0, 4, 0]), Some([0, 0, 0]));
    assert_eq!(EmitterEntry::local([31, 95, 31], [0, 4, 0]), Some([47, 47, 47]));
    assert_eq!(EmitterEntry::local([32, 64, 0], [0, 4, 0]), None);
    assert_eq!(EmitterEntry::local([-17, 64, 0], [0, 4, 0]), None);
    assert_eq!(EmitterEntry::local([0, 0, 0], [i32::MIN, 0, 0]), None);
}

#[test]
fn entry_layout() {
    let entry = EmitterEntry::bytes(0x1122_3344_5566_7788, [1, 2, 3], 20);
    assert_eq!(&entry[..8], &0x1122_3344_5566_7788u64.to_le_bytes());
    assert_eq!(&entry[8..12], &[1, 2, 3, 15]);
    assert!(entry[12..].iter().all(|byte| *byte == 0));
}

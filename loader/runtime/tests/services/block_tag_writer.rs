use amml_runtime::{BlockTagError, BlockTagValue, BlockTagWriter};

// A named entry: the tag type byte, the u16 little-endian name length and the name bytes.
fn named(kind: u8, name: &str) -> Vec<u8> {
    let mut bytes = vec![kind];
    bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
    bytes.extend_from_slice(name.as_bytes());
    bytes
}

fn text(value: &str) -> Vec<u8> {
    let mut bytes = (value.len() as u16).to_le_bytes().to_vec();
    bytes.extend_from_slice(value.as_bytes());
    bytes
}

#[test]
fn writes_name_states_and_version() {
    let bytes = BlockTagWriter::new().write("minecraft:stone", &[], 0x0102_0304).unwrap();

    let mut expected = named(10, "");
    expected.extend(named(8, "name"));
    expected.extend(text("minecraft:stone"));
    expected.extend(named(10, "states"));
    expected.push(0);
    expected.extend(named(3, "version"));
    expected.extend_from_slice(&0x0102_0304i32.to_le_bytes());
    expected.push(0);
    assert_eq!(bytes, expected);
}

#[test]
fn int_bool_and_text_states_have_their_tags() {
    let states = [
        ("age", BlockTagValue::Int(-3)),
        ("open_bit", BlockTagValue::Bool(true)),
        ("facing", BlockTagValue::Text("north")),
        ("lit", BlockTagValue::Bool(false)),
    ];

    let bytes = BlockTagWriter::new().write("minecraft:x", &states, 7).unwrap();

    let mut expected = named(10, "");
    expected.extend(named(8, "name"));
    expected.extend(text("minecraft:x"));
    expected.extend(named(10, "states"));
    expected.extend(named(3, "age"));
    expected.extend_from_slice(&(-3i32).to_le_bytes());
    expected.extend(named(1, "open_bit"));
    expected.push(1);
    expected.extend(named(8, "facing"));
    expected.extend(text("north"));
    expected.extend(named(1, "lit"));
    expected.push(0);
    expected.push(0);
    expected.extend(named(3, "version"));
    expected.extend_from_slice(&7i32.to_le_bytes());
    expected.push(0);
    assert_eq!(bytes, expected);
}

#[test]
fn escapes_nothing_and_keeps_utf8() {
    let states = [("na\"me\\é", BlockTagValue::Text("ü\n\0x"))];

    let bytes = BlockTagWriter::new().write("mod:blöck\"", &states, 1).unwrap();

    let mut expected = named(10, "");
    expected.extend(named(8, "name"));
    expected.extend(text("mod:blöck\""));
    expected.extend(named(10, "states"));
    expected.extend(named(8, "na\"me\\é"));
    expected.extend(text("ü\n\0x"));
    expected.push(0);
    expected.extend(named(3, "version"));
    expected.extend_from_slice(&1i32.to_le_bytes());
    expected.push(0);
    assert_eq!(bytes, expected);
}

#[test]
fn a_text_longer_than_the_length_field_is_refused() {
    let long = "a".repeat(usize::from(u16::MAX) + 1);
    let states = [("facing", BlockTagValue::Text(&long))];

    assert_eq!(BlockTagWriter::new().write(&long, &[], 1), Err(BlockTagError::TooLong { len: long.len() }));
    assert_eq!(BlockTagWriter::new().write("minecraft:x", &states, 1), Err(BlockTagError::TooLong { len: long.len() }));
}

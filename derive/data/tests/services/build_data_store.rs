use amml_data::{BuildData, BuildDataStore, BuildId, DataError, EntryId, EntryKind, Fingerprint, Resolved};
use tempfile::TempDir;

#[test]
fn written_file_matches_the_stable_format() {
    let mut data = BuildData::new(BuildId::new(0x68f0c1a2, 0x0123456789abcdef));
    data.entries
        .insert(EntryId("b.fn".to_string()), Resolved { kind: EntryKind::Function, value: 0x20, recipes_agreeing: 2 });
    data.entries
        .insert(EntryId("a.field".to_string()), Resolved { kind: EntryKind::Field, value: 0x5a8, recipes_agreeing: 1 });
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("build.hcl");
    BuildDataStore::write(&path, &data).unwrap();

    let expected = [
        "format = 2",
        "",
        "build {",
        "  timestamp = \"0x68f0c1a2\"",
        "  text_hash = \"0x0123456789abcdef\"",
        "}",
        "",
        "resolved \"a.field\" {",
        "  kind        = \"field\"",
        "  value       = \"0x5a8\"",
        "  recipes     = 1",
        "}",
        "",
        "resolved \"b.fn\" {",
        "  kind        = \"function\"",
        "  value       = \"0x20\"",
        "  recipes     = 2",
        "}",
        "",
    ]
    .join("\n");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
}

struct Fixture;

impl Fixture {
    fn read(dir: &TempDir, text: &str) -> Result<BuildData, DataError> {
        let path = dir.path().join("build.hcl");
        assert!(std::fs::write(&path, text).is_ok());
        BuildDataStore::read(&path)
    }
}

const HEADER: &str = "format = 1\nbuild {\n  timestamp = \"0x68f0c1a2\"\n  text_hash = \"0x0123456789abcdef\"\n}\n";
const RESOLVED: &str = "\nresolved \"a\" {\n  kind    = \"field\"\n  value   = \"0x5a8\"\n  recipes = 1\n}\n";

#[test]
fn read_parses_a_fixed_file() {
    let data = Fixture::read(&TempDir::new().unwrap(), &format!("{HEADER}{RESOLVED}")).unwrap();
    assert_eq!(data.build, BuildId::new(0x68f0c1a2, 0x0123456789abcdef));
    let resolved = data.entries.get(&EntryId("a".to_string())).unwrap();
    assert_eq!(*resolved, Resolved { kind: EntryKind::Field, value: 0x5a8, recipes_agreeing: 1 });
    assert_eq!(data.entries.len(), 1);
}

#[test]
fn read_rejects_a_duplicate_build_block() {
    let build_only = HEADER.replace("format = 1\n", "");
    let err = Fixture::read(&TempDir::new().unwrap(), &format!("{HEADER}{build_only}")).unwrap_err();
    assert!(matches!(err, DataError::Parse { message, .. } if message.contains("duplicate")));
}

#[test]
fn read_rejects_a_duplicate_resolved_id() {
    let err = Fixture::read(&TempDir::new().unwrap(), &format!("{HEADER}{RESOLVED}{RESOLVED}")).unwrap_err();
    assert!(
        matches!(err, DataError::Parse { entry, message, .. } if entry.as_deref() == Some("a") && message.contains("duplicate"))
    );
}

#[test]
fn read_rejects_a_missing_format() {
    let text = HEADER.replace("format = 1\n", "");
    let err = Fixture::read(&TempDir::new().unwrap(), &text).unwrap_err();
    assert!(matches!(err, DataError::Parse { message, .. } if message.contains("missing `format`")));
}

#[test]
fn read_rejects_an_unknown_format_and_names_both_versions() {
    let text = HEADER.replace("format = 1", "format = 3");
    let err = Fixture::read(&TempDir::new().unwrap(), &text).unwrap_err();
    assert!(matches!(err, DataError::Parse { message, .. }
        if message.contains("unsupported data format 3") && message.contains("format 1") && message.contains("format 2")));
}

#[test]
fn write_refuses_an_id_the_reader_could_not_take_back() {
    let mut data = BuildData::new(BuildId::new(1, 2));
    data.entries.insert(
        EntryId("bad\"id".to_string()),
        Resolved { kind: EntryKind::Function, value: 0x20, recipes_agreeing: 1 },
    );
    let dir = TempDir::new().unwrap();
    let err = BuildDataStore::write(&dir.path().join("build.hcl"), &data).unwrap_err();
    assert!(matches!(err, DataError::Invalid { entry, .. } if entry.0 == "bad\"id"));
}

#[test]
fn read_rejects_extra_labels() {
    let text = format!("{HEADER}\nresolved \"a\" \"b\" {{\n  kind = \"field\"\n  value = \"0x1\"\n  recipes = 1\n}}\n");
    assert!(Fixture::read(&TempDir::new().unwrap(), &text).is_err());
    let text = HEADER.replace("build {", "build \"x\" {");
    assert!(Fixture::read(&TempDir::new().unwrap(), &text).is_err());
}

fn one_entry(value: u64) -> BuildData {
    let mut data = BuildData::new(BuildId::new(1, 2));
    data.entries.insert(EntryId("a".to_string()), Resolved { kind: EntryKind::Function, value, recipes_agreeing: 1 });
    data
}

fn names(dir: &TempDir) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir.path())
        .into_iter()
        .flatten()
        .flatten()
        .map(|item| item.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn write_replaces_an_existing_file_and_leaves_no_temporary_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("build.hcl");
    BuildDataStore::write(&path, &one_entry(0x10)).unwrap();

    BuildDataStore::write(&path, &one_entry(0x20)).unwrap();

    let data = BuildDataStore::read(&path).unwrap();
    assert_eq!(data.entries.get(&EntryId("a".to_string())).map(|resolved| resolved.value), Some(0x20));
    assert_eq!(names(&dir), ["build.hcl"]);
}

#[test]
fn a_write_that_cannot_replace_the_target_leaves_no_temporary_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("build.hcl");
    std::fs::create_dir(&path).unwrap();

    let result = BuildDataStore::write(&path, &one_entry(0x10));

    assert!(matches!(result, Err(DataError::Io { .. })));
    assert_eq!(names(&dir), ["build.hcl"]);
}

fn id(text: &str) -> EntryId {
    EntryId(text.to_string())
}

fn mixed_data() -> BuildData {
    let mut data = BuildData::new(BuildId::new(0x68f0c1a2, 0x0123456789abcdef));
    data.entries.insert(id("a.fn"), Resolved { kind: EntryKind::Function, value: 0x20, recipes_agreeing: 2 });
    data.fingerprints.insert(id("a.fn"), Fingerprint(1));
    data.failed.insert(id("b.vtable"), "all recipes failed: no match".to_string());
    data.fingerprints.insert(id("b.vtable"), Fingerprint(2));
    data
}

#[test]
fn format_2_keeps_fingerprints_and_failures() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("build.hcl");
    let data = mixed_data();
    BuildDataStore::write(&path, &data).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(
        "format = 2
"
    ));
    assert!(text.contains(
        "  fingerprint = \"0000000000000001\"
"
    ));
    assert!(text.contains(
        "failed \"b.vtable\" {
"
    ));
    assert!(text.contains(
        "  fingerprint = \"0000000000000002\"
"
    ));
    assert!(text.contains(
        "  reason      = \"all recipes failed: no match\"
"
    ));
    let back = BuildDataStore::read(&path).unwrap();
    assert_eq!(back, data);
}

#[test]
fn format_1_reads_without_fingerprints() {
    let data = Fixture::read(&TempDir::new().unwrap(), &format!("{HEADER}{RESOLVED}")).unwrap();
    assert_eq!(data.entries.len(), 1);
    assert!(data.fingerprints.is_empty());
    assert!(data.failed.is_empty());
}

#[test]
fn a_resolved_entry_without_a_fingerprint_line_is_accepted() {
    let text = format!("{}{RESOLVED}", HEADER.replace("format = 1", "format = 2"));
    let data = Fixture::read(&TempDir::new().unwrap(), &text).unwrap();
    assert_eq!(data.entries.len(), 1);
    assert!(data.fingerprints.is_empty());
}

#[test]
fn the_written_text_is_byte_stable() {
    let dir = TempDir::new().unwrap();
    let first = dir.path().join("first.hcl");
    let second = dir.path().join("second.hcl");
    BuildDataStore::write(&first, &mixed_data()).unwrap();
    BuildDataStore::write(&second, &mixed_data()).unwrap();
    assert_eq!(std::fs::read(&first).unwrap(), std::fs::read(&second).unwrap());
}

#[test]
fn read_rejects_an_id_that_is_both_resolved_and_failed() {
    let text = format!(
        "{}{RESOLVED}
failed \"a\" {{
  reason = \"x\"
}}
",
        HEADER.replace("format = 1", "format = 2")
    );
    let err = Fixture::read(&TempDir::new().unwrap(), &text).unwrap_err();
    assert!(matches!(err, DataError::Parse { message, .. } if message.contains("duplicate")));
}

#[test]
fn read_rejects_a_malformed_fingerprint() {
    let text = format!(
        "{}
resolved \"a\" {{
  kind = \"field\"
  value = \"0x1\"
  recipes = 1
  fingerprint = \"xyz\"
}}
",
        HEADER.replace("format = 1", "format = 2")
    );
    assert!(Fixture::read(&TempDir::new().unwrap(), &text).is_err());
}

fn round_trip_reason(reason: &str) -> String {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("build.hcl");
    let mut data = BuildData::new(BuildId::new(1, 2));
    data.failed.insert(id("a"), reason.to_string());
    BuildDataStore::write(&path, &data).unwrap();
    BuildDataStore::read(&path).unwrap().failed.remove(&id("a")).unwrap()
}

#[test]
fn a_reason_with_template_starts_reads_back_equal() {
    for reason in ["has ${x} in it", "has %{x} in it", "$${x} and %%{x}", "ends with $", "100% {not a template}"] {
        assert_eq!(round_trip_reason(reason), reason);
    }
}

#[test]
fn a_reason_with_control_characters_reads_back_equal() {
    for reason in ["tab\there", "line\nbreak\r\n", "bell\u{7} nul\u{0} del\u{7f}", "quote \" and \\ slash"] {
        assert_eq!(round_trip_reason(reason), reason);
    }
}

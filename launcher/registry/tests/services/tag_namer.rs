use amml_registry::{Clock, RegistryError, TagNamer};

fn tags(list: &[&str]) -> Vec<String> {
    list.iter().map(|tag| tag.to_string()).collect()
}

#[test]
fn first_and_next_tags() {
    assert_eq!(TagNamer::next("2026-11-02", &[]).unwrap(), "2026.11.02-1");
    assert_eq!(TagNamer::next("2026-11-02", &tags(&["2026.11.02-1", "2026.11.01-3"])).unwrap(), "2026.11.02-2");
    assert_eq!(TagNamer::next("2026-11-02", &tags(&["2026.11.02-1", "2026.11.02-3"])).unwrap(), "2026.11.02-2");
}

#[test]
fn a_tag_is_never_reused_even_with_unrelated_tags() {
    let existing = tags(&["2026.11.02-1", "2026.11.02-2", "v1", "2026.11.02-x", "2026.11.020-3"]);
    assert_eq!(TagNamer::next("2026-11-02", &existing).unwrap(), "2026.11.02-3");
}

#[test]
fn a_date_in_another_shape_is_refused() {
    for date in ["2026.11.02", "26-11-02", "2026-1-2", ""] {
        assert!(matches!(TagNamer::next(date, &[]), Err(RegistryError::Usage(_))), "{date}");
    }
}

#[test]
fn clock_formats_unix_seconds_as_utc() {
    assert_eq!(Clock::format(0), "1970-01-01T00:00:00Z");
    assert_eq!(Clock::format(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(Clock::format(1_762_077_599), "2025-11-02T09:59:59Z");
}

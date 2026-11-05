use amml_gdk::Timestamp;

#[test]
fn dates_are_right_around_leap_days_and_century_years() {
    assert_eq!(Timestamp::format(0), "1970-01-01T00:00:00Z");
    assert_eq!(Timestamp::format(951_782_400), "2000-02-29T00:00:00Z");
    assert_eq!(Timestamp::format(1_709_164_800 + 3_661), "2024-02-29T01:01:01Z");
    assert_eq!(Timestamp::format(1_709_251_200), "2024-03-01T00:00:00Z");
    assert_eq!(Timestamp::format(4_102_444_799), "2099-12-31T23:59:59Z");
}

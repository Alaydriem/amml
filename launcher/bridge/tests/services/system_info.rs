use amml_launcher_bridge::SystemInfo;

#[test]
fn the_windows_version_is_the_number_after_version() {
    assert_eq!(
        SystemInfo::version_in("\r\nMicrosoft Windows [Version 10.0.26300.7171]\r\n").as_deref(),
        Some("10.0.26300.7171")
    );
    assert_eq!(SystemInfo::version_in("Microsoft Windows"), None);
    assert_eq!(SystemInfo::version_in("Version unknown"), None);
}

#[test]
fn a_localised_windows_that_prints_version_in_lower_case_is_still_read() {
    assert_eq!(
        SystemInfo::version_in("Microsoft Windows [version 10.0.26300.7171]").as_deref(),
        Some("10.0.26300.7171")
    );
}

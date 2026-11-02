use amml_gdk::DeveloperMode;

const ON: &str = "\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock\r\n    AllowDevelopmentWithoutDevLicense    REG_DWORD    0x1\r\n";

#[test]
fn reads_the_registry_value_from_query_output() {
    assert!(DeveloperMode::is_on_in(ON));
    assert!(!DeveloperMode::is_on_in(&ON.replace("0x1", "0x0")));
    assert!(!DeveloperMode::is_on_in(""));
    assert!(!DeveloperMode::is_on_in("ERROR: The system was unable to find the specified registry key or value."));
}

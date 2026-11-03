use amml_launcher_core::{WindowProbe, WindowsWindowProbe};

#[test]
fn a_test_process_and_an_unused_pid_have_no_visible_window() {
    let probe = WindowsWindowProbe::new();
    assert!(!probe.has_window(std::process::id()));
    assert!(!probe.has_window(0xFFFF_FFF0));
}

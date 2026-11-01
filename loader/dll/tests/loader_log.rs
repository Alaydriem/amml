use amml::LoaderLog;

use crate::support::Harness;

#[test]
fn a_repeat_install_keeps_the_first_sink_and_opens_no_files() {
    let harness = Harness::start();
    let other = harness.root.path().join("other-logs");

    LoaderLog::install(&other).unwrap();

    assert!(!other.exists());
    assert!(harness.logs().exists());
}

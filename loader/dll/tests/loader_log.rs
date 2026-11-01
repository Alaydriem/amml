use amml::LoaderLog;
use amml_abi::KnownLogLevel;

use crate::support::{Harness, LogFile};

#[test]
fn a_repeat_install_keeps_the_first_sink_and_opens_no_files() {
    let harness = Harness::start();
    let other = harness.root.path().join("other-logs");

    LoaderLog::install(&other).unwrap();

    assert!(!other.exists());
    assert!(harness.logs().exists());
}

#[test]
fn a_registered_mod_logs_to_its_namespace_folder() {
    let harness = Harness::start();
    LoaderLog::register_mod("@acme/demo", "acme", "demo");

    LoaderLog::module_line("@acme/demo", KnownLogLevel::Info, "hello from demo");

    let text = LogFile::wait_for(&harness.logs().join("@acme").join("demo.log"), "hello from demo");
    assert!(text.contains("hello from demo"), "{text}");
}

#[test]
fn an_unregistered_module_goes_to_the_loader_log() {
    let harness = Harness::start();

    LoaderLog::module_line("alaydriem-zoom", KnownLogLevel::Info, "from a stray module");

    let text = LogFile::wait_for(&harness.logs().join("loader.log"), "from a stray module");
    assert!(text.contains("from a stray module"), "{text}");
    assert!(!harness.logs().join("alaydriem-zoom.log").exists());
    assert!(!harness.logs().join("alaydriem-zoom").exists());
}

use amml::FolderRedirect;
use tempfile::tempdir;

use crate::support::FolderProbe;

#[test]
fn the_crt_and_ansi_routes_are_hooked_in_the_trace_build() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();

    // SAFETY: the probe module stays mapped for the life of the test process.
    let redirect = unsafe { FolderRedirect::install(probe.base(), dir.path()) }.unwrap();

    for function in
        ["GetTempPathW", "GetTempPathA", "GetEnvironmentVariableA", "getenv", "getenv_s", "_dupenv_s", "_wdupenv_s"]
    {
        assert!(redirect.slots().iter().any(|slot| slot.function() == function), "{function} was not hooked");
    }
}

#[test]
fn the_traced_routes_pass_through_to_the_real_functions() {
    let probe = FolderProbe::load();
    let dir = tempdir().unwrap();
    let real_a = probe.env_a("APPDATA");
    let real_temp_a = probe.temp_path_a();
    let real_getenv = probe.getenv("APPDATA");
    let real_getenv_s = probe.getenv_s("APPDATA");
    let real_dupenv_s = probe.dupenv_s("APPDATA");
    let real_wdupenv_s = probe.wdupenv_s("APPDATA");
    let real_unset = probe.getenv("AMML_SURELY_UNSET_VARIABLE");
    // SAFETY: the probe module stays mapped for the life of the test process.
    let _redirect = unsafe { FolderRedirect::install(probe.base(), dir.path()) }.unwrap();

    assert_eq!(probe.env_a("APPDATA"), real_a);
    assert_eq!(probe.temp_path_a(), real_temp_a);
    assert_eq!(probe.getenv("APPDATA"), real_getenv);
    assert_eq!(probe.getenv_s("APPDATA"), real_getenv_s);
    assert_eq!(probe.dupenv_s("APPDATA"), real_dupenv_s);
    assert_eq!(probe.wdupenv_s("APPDATA"), real_wdupenv_s);
    assert_eq!(probe.getenv("AMML_SURELY_UNSET_VARIABLE"), real_unset);
    assert!(real_getenv.is_some_and(|value| !value.is_empty()));
}

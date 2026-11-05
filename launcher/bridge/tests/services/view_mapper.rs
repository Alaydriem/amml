use crate::support::Samples;
use amml_launcher_bridge::{
    DataModeView, ModOriginView, NeedsAmmlView, RegistryStatusView, UnixMillis, UpdateStatusView, ViewMapper,
};
use amml_launcher_core::{
    LauncherSettings, ModSource, NeedsAmml, RegistryRef, RegistryStatus, RunningGame, UpdateRow, UpdateStatus,
};
use amml_registry::{AmmlSection, ApiSummary, FileRef, IndexSummary, RegistryRoot};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn row(id: &str, status: UpdateStatus, available: Option<&str>) -> UpdateRow {
    UpdateRow {
        id: id.to_string(),
        installed: Some("1.0.0".to_string()),
        available: available.map(str::to_string),
        status,
        api_change: false,
        needs_amml: None,
    }
}

fn root(mod_count: u32, api_two_mods: u32) -> RegistryRoot {
    let mut apis = BTreeMap::new();
    apis.insert("2".to_string(), ApiSummary { mods: api_two_mods, pages: 1 });
    RegistryRoot {
        schema: 2,
        name: "AMML".to_string(),
        published: "2026-11-01".to_string(),
        mod_count,
        amml: AmmlSection { repo: "alaydriem/amml".to_string(), versions: Vec::new() },
        index: IndexSummary { page_size: 100, apis },
        names: FileRef { path: "names.json".to_string(), sha256: String::new(), size: 0 },
    }
}

#[test]
fn an_instance_with_a_data_dir_inside_it_is_own_data() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let settings = LauncherSettings::new(Vec::new(), false, None);
    let mut state = Samples::instance(dir.path(), "own");
    state.data_dir = Some(state.folder.join("data"));

    let own = ViewMapper::instance(&state, &settings, &paths, &[], 0, None);
    assert_eq!(own.data_mode, DataModeView::Own);
    assert_eq!(own.data_dir, Some(state.folder.join("data").display().to_string()));

    state.data_dir = Some(dir.path().join("data"));
    let shared = ViewMapper::instance(&state, &settings, &paths, &[], 0, None);
    assert_eq!(shared.data_mode, DataModeView::Shared);
}

#[test]
fn an_instance_without_a_redirect_shows_the_launcher_data_folder() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let settings = LauncherSettings::new(Vec::new(), false, None);
    let state = Samples::instance(dir.path(), "plain");

    let view = ViewMapper::instance(&state, &settings, &paths, &[], 2, None);

    assert_eq!(view.data_mode, DataModeView::Shared);
    assert_eq!(view.data_dir, Some(dir.path().join("data").display().to_string()));
    assert_eq!(view.update_count, 2);
}

#[test]
fn an_instance_running_from_this_launcher_has_its_start_time() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let settings = LauncherSettings::new(Vec::new(), false, None);
    let state = Samples::instance(dir.path(), "play");
    let started = UNIX_EPOCH + Duration::from_millis(1_700_000_123_000);
    let running = [
        RunningGame { pid: 7, instance: "other".to_string(), started: UNIX_EPOCH },
        RunningGame { pid: 9, instance: "play".to_string(), started },
    ];

    let view = ViewMapper::instance(&state, &settings, &paths, &running, 0, Some(started));
    let idle = ViewMapper::instance(&Samples::instance(dir.path(), "idle"), &settings, &paths, &running, 0, None);

    assert_eq!(view.running_since, Some(1_700_000_123_000));
    assert_eq!(view.last_played, Some(UnixMillis::of(started)));
    assert_eq!(idle.running_since, None);
    assert_eq!(idle.last_played, None);
}

#[test]
fn the_amml_row_comes_first_and_is_core() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = Samples::instance(dir.path(), "play");
    state.mods.push(Samples::installed("@alaydriem/peek", Some("1.0.0"), ModSource::Registry("https://r/".into())));
    let rows = [
        row("alaydriem/amml", UpdateStatus::Available, Some("26.60.2")),
        row("@alaydriem/peek", UpdateStatus::UpToDate, None),
    ];

    let views = ViewMapper::mods(&state, &rows, &["@alaydriem/peek".to_string()]);

    assert_eq!(views[0].id, "alaydriem/amml");
    assert_eq!(views[0].origin, ModOriginView::Core);
    assert_eq!(views[0].installed_version.as_deref(), Some("26.60.1"));
    assert_eq!(views[0].available_version.as_deref(), Some("26.60.2"));
    assert_eq!(views[0].update, UpdateStatusView::Available);
    assert_eq!(views[1].id, "@alaydriem/peek");
    assert_eq!(views[1].origin, ModOriginView::Core);
    assert_eq!(views[1].update, UpdateStatusView::UpToDate);
}

#[test]
fn a_mod_with_a_url_source_is_url_origin_and_unofficial() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = Samples::instance(dir.path(), "play");
    state.mods.push(Samples::installed("@x/night", Some("1.2.0"), ModSource::Url("https://x/n.zip".into())));
    state.mods.push(Samples::installed("@x/mine", Some("1.0.0"), ModSource::Local));

    let views = ViewMapper::mods(&state, &[], &[]);

    assert_eq!(views[1].origin, ModOriginView::Url);
    assert_eq!(views[1].update, UpdateStatusView::Unofficial);
    assert_eq!(views[1].registry_base, None);
    assert_eq!(views[2].origin, ModOriginView::Local);
}

#[test]
fn a_mod_with_no_version_is_unknown_not_up_to_date() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = Samples::instance(dir.path(), "play");
    state.mods.push(Samples::installed("@x/built", None, ModSource::Registry("https://r/".into())));
    let rows = [row("@x/built", UpdateStatus::UpToDate, None)];

    let with_row = ViewMapper::mods(&state, &rows, &[]);
    let without_row = ViewMapper::mods(&state, &[], &[]);

    assert_eq!(with_row[1].update, UpdateStatusView::Unknown);
    assert_eq!(without_row[1].update, UpdateStatusView::Unknown);
}

#[test]
fn a_stale_registry_keeps_its_cached_mod_count() {
    let base = RegistryRef::new("https://example.org/reg/".to_string());
    let cached = root(7, 5);
    let status = RegistryStatus::Stale { reason: "timed out".to_string() };

    let all = ViewMapper::registry(&base, Some(&cached), &status, None, None);
    let for_api = ViewMapper::registry(&base, Some(&cached), &status, Some(2), None);
    let other_api = ViewMapper::registry(&base, Some(&cached), &status, Some(9), None);
    let offline = ViewMapper::registry(&base, None, &RegistryStatus::Offline, None, None);

    assert_eq!(all.status, RegistryStatusView::Stale);
    assert_eq!(all.mod_count, Some(7));
    assert_eq!(for_api.mod_count, Some(5));
    assert_eq!(other_api.mod_count, Some(0));
    assert_eq!(offline.mod_count, None);
    assert_eq!(offline.status, RegistryStatusView::Offline);
    assert!(!all.official);
}

#[test]
fn the_official_registry_is_marked_official() {
    let view = ViewMapper::registry(
        &LauncherSettings::official(),
        None,
        &RegistryStatus::Ok,
        None,
        Some(SystemTime::UNIX_EPOCH),
    );
    assert!(view.official);
    assert_eq!(view.fetched, Some(0));
}

#[test]
fn update_status_maps_local_build() {
    let dir = tempfile::tempdir().unwrap();
    let state = Samples::instance(dir.path(), "play");
    let rows = [row("alaydriem/amml", UpdateStatus::LocalBuild, Some("26.60.5"))];

    let views = ViewMapper::mods(&state, &rows, &[]);

    assert_eq!(views[0].update, UpdateStatusView::LocalBuild);
    assert_eq!(views[0].available_version.as_deref(), Some("26.60.5"));
}

#[test]
fn instance_view_carries_amml_local() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let settings = LauncherSettings::new(Vec::new(), false, None);
    let mut state = Samples::instance(dir.path(), "loc");
    state.amml.version = Some("26.60.5+local".to_string());
    state.amml.local = true;

    let local = ViewMapper::instance(&state, &settings, &paths, &[], 0, None);
    state.amml.local = false;
    let release = ViewMapper::instance(&state, &settings, &paths, &[], 0, None);

    assert!(local.amml_local);
    assert!(!release.amml_local);
}

#[test]
fn update_status_maps_needs_amml() {
    assert_eq!(ViewMapper::update_status(UpdateStatus::NeedsAmml), UpdateStatusView::NeedsAmml);
}

#[test]
fn mod_view_carries_needs_amml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = Samples::instance(dir.path(), "play");
    state.mods.push(Samples::installed("@alaydriem/peek", Some("1.0.0"), ModSource::Registry("https://r/".into())));
    state.mods.push(Samples::installed("@alaydriem/calm", Some("1.0.0"), ModSource::Registry("https://r/".into())));
    let mut blocked = row("@alaydriem/peek", UpdateStatus::NeedsAmml, None);
    blocked.needs_amml = Some(NeedsAmml { version: "26.62.0".to_string(), game: Some("1.26.10".to_string()) });
    let mut later = row("@alaydriem/calm", UpdateStatus::Available, Some("1.1.0"));
    later.needs_amml = Some(NeedsAmml { version: "26.63.0".to_string(), game: None });

    let views = ViewMapper::mods(&state, &[blocked, later], &[]);

    assert_eq!(views[0].needs_amml, None);
    assert_eq!(views[1].update, UpdateStatusView::NeedsAmml);
    assert_eq!(
        views[1].needs_amml,
        Some(NeedsAmmlView { version: "26.62.0".to_string(), game: Some("1.26.10".to_string()) })
    );
    assert_eq!(views[2].update, UpdateStatusView::Available);
    assert_eq!(views[2].needs_amml, Some(NeedsAmmlView { version: "26.63.0".to_string(), game: None }));
}

use crate::support::{Harness, InstanceFolder};
use amml_launcher_core::{CliArgs, LauncherCli};
use clap::Parser;
use std::path::PathBuf;

// runs the command line over the harness root and its fake official registry
fn run(harness: &Harness, args: &[&str]) -> (i32, String) {
    let root = harness.root.path().to_string_lossy().to_string();
    let official = harness.server.base();
    let mut all = vec!["amml-launcher", "--root", root.as_str(), "--official", official.as_str()];
    all.extend_from_slice(args);
    let parsed = CliArgs::try_parse_from(all).unwrap();
    let mut out = Vec::new();
    let code = LauncherCli::run(parsed, &mut out);
    (code, String::from_utf8(out).unwrap())
}

fn instance(harness: &Harness, name: &str) -> PathBuf {
    let dir = harness.paths.instance(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn instance_list_prints_one_line_per_instance_with_amml_and_mod_count() {
    let harness = Harness::new();
    let one = instance(&harness, "one");
    InstanceFolder::create(&one, "1.26.60.05")
        .with_amml(Some("26.60.1"))
        .with_mod(true, "a", "x", Some("1.0.0"), 2)
        .with_mod(false, "a", "y", Some("1.0.0"), 2);
    let two = instance(&harness, "two");
    InstanceFolder::create(&two, "1.26.52.03");
    std::fs::create_dir_all(harness.paths.instance("not-an-instance")).unwrap();

    let (code, out) = run(&harness, &["instance", "list"]);
    assert_eq!(code, 0, "{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"one  1.26.60.05  amml 26.60.1  mods 2"), "{out}");
    assert!(lines.contains(&"two  1.26.52.03  amml none  mods 0"), "{out}");
    assert_eq!(lines.iter().filter(|line| line.starts_with("skipped:")).count(), 1, "{out}");

    let (code, out) = run(&harness, &["instance", "show", "one"]);
    assert_eq!(code, 0);
    assert!(out.contains("@a/x  1.0.0  on  local") && out.contains("@a/y  1.0.0  off  local"), "{out}");
}

#[test]
fn mod_off_on_a_core_mod_exits_2_and_names_it() {
    let harness = Harness::new();
    let docs = [
        harness.mod_document("amml", "modmanager", true, false, "26.60.1", 2),
        harness.mod_document("alaydriem", "zoom", false, true, "26.60.1", 2),
    ];
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &docs);
    let dir = instance(&harness, "one");
    let folder = InstanceFolder::create(&dir, "1.26.60.05")
        .with_amml(Some("26.60.1"))
        .with_mod(true, "amml", "modmanager", Some("26.60.1"), 2)
        .with_mod(true, "alaydriem", "zoom", Some("26.60.1"), 2);
    let before = InstanceFolder::hashes(&dir);

    let (code, out) = run(&harness, &["mod", "off", "one", "@amml/modmanager"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.starts_with("error:") && out.contains("@amml/modmanager") && out.contains("core"), "{out}");
    let (code, out) = run(&harness, &["mod", "remove", "one", "@amml/modmanager"]);
    assert_eq!(code, 2, "{out}");
    assert_eq!(InstanceFolder::hashes(&dir), before);

    let (code, out) = run(&harness, &["mod", "off", "one", "@alaydriem/zoom"]);
    assert_eq!(code, 0, "{out}");
    assert!(folder.mod_folder(false, "alaydriem", "zoom").is_dir());
    let (code, out) = run(&harness, &["mod", "on", "one", "@alaydriem/zoom"]);
    assert_eq!(code, 0, "{out}");
    assert!(folder.mod_folder(true, "alaydriem", "zoom").is_dir());
    assert!(folder.mod_folder(true, "alaydriem", "zoom").join("config").join("keys.hcl").is_file());
}

#[test]
fn delete_without_the_matching_confirm_exits_1_and_keeps_the_folder() {
    let harness = Harness::new();
    let dir = instance(&harness, "one");
    InstanceFolder::create(&dir, "1.26.60.05");
    let (code, out) = run(&harness, &["instance", "delete", "one", "--confirm", "two"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.starts_with("error:"), "{out}");
    assert!(dir.is_dir());

    let (code, out) = run(&harness, &["instance", "delete", "one", "--confirm", "one"]);
    assert_eq!(code, 0, "{out}");
    assert!(!dir.exists());
}

#[test]
fn updates_prints_available_unknown_and_unofficial_rows() {
    let harness = Harness::new();
    let docs = [harness.mod_document("a", "newer", false, true, "26.60.2", 2)];
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &docs);
    let dir = instance(&harness, "one");
    let base = harness.server.base();
    InstanceFolder::create(&dir, "1.26.60.05")
        .with_amml(Some("26.60.1"))
        .with_mod(true, "a", "newer", Some("26.60.0"), 2)
        .with_mod(true, "a", "built", None, 2)
        .with_mod(true, "a", "url", Some("26.60.0"), 2)
        .with_version_json(&format!(
            "{{\"gameVersion\":\"1.26.60.05\",\"launcher\":{{\"amml\":\"26.60.1\",\"mods\":{{\"@a/newer\":{{\"version\":\"26.60.0\",\"source\":\"{base}\"}},\"@a/url\":{{\"version\":\"26.60.0\",\"source\":\"url:https://x.example/u.zip\"}}}}}}}}"
        ));

    let (code, out) = run(&harness, &["updates", "one"]);
    assert_eq!(code, 0, "{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"alaydriem/amml  26.60.1  26.60.1  up-to-date"), "{out}");
    assert!(lines.contains(&"@a/newer  26.60.0  26.60.2  available"), "{out}");
    assert!(lines.contains(&"@a/built  -  -  unknown"), "{out}");
    assert!(lines.contains(&"@a/url  26.60.0  -  unofficial"), "{out}");
}

#[test]
fn registry_commands_read_the_official_registry() {
    let harness = Harness::new();
    let docs = [harness.mod_document("alaydriem", "zoom", false, true, "26.60.1", 2)];
    harness.publish(vec![harness.amml_release("26.60.1", 2, &["1.26.60.5"])], &docs);

    let (code, out) = run(&harness, &["registry", "status"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Official") && out.contains(": ok") && out.contains("26.60.1 (API 2)"), "{out}");
    let (code, out) = run(&harness, &["registry", "search", "zoom"]);
    assert_eq!(code, 0);
    assert!(out.contains("@alaydriem/zoom  Zoom  (API 2)"), "{out}");
    let (code, out) = run(&harness, &["registry", "show", "@alaydriem/zoom"]);
    assert_eq!(code, 0);
    assert!(out.contains("26.60.1  API 2"), "{out}");
    let (code, out) = run(&harness, &["registry", "show", "@alaydriem/none"]);
    assert_eq!(code, 1);
    assert!(out.starts_with("error:"), "{out}");
}

#[test]
fn settings_add_and_remove_registries_and_never_remove_the_official_one() {
    let harness = Harness::new();
    let (code, out) = run(&harness, &["settings", "add-registry", "http://insecure.example/"]);
    assert_eq!(code, 1, "{out}");
    let (code, out) = run(&harness, &["settings", "add-registry", "https://example.org/reg"]);
    assert_eq!(code, 0, "{out}");
    let (_, shown) = run(&harness, &["settings", "show"]);
    assert!(
        shown.contains("registry https://alaydriem.github.io/amml-registry/")
            && shown.contains("registry https://example.org/reg/"),
        "{shown}"
    );
    let (code, out) = run(&harness, &["settings", "remove-registry", "https://alaydriem.github.io/amml-registry/"]);
    assert_eq!(code, 1, "{out}");
    let (code, _) = run(&harness, &["settings", "remove-registry", "https://example.org/reg/"]);
    assert_eq!(code, 0);
    let (code, _) = run(&harness, &["settings", "preview", "on"]);
    assert_eq!(code, 0);
    let (_, shown) = run(&harness, &["settings", "show"]);
    assert!(shown.contains("preview on") && !shown.contains("example.org"), "{shown}");
}

#[test]
fn show_and_launch_refuse_a_name_that_leaves_the_instances_folder() {
    let harness = Harness::new();
    let outside = harness.root.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    InstanceFolder::create(&outside, "1.26.60.05");
    for command in [["instance", "show", "..\\outside"], ["launch", "..\\outside", "--window-timeout"]] {
        let mut args = command.to_vec();
        if args.last() == Some(&"--window-timeout") {
            args.push("1");
        }
        let (code, out) = run(&harness, &args);
        assert_eq!(code, 1, "{out}");
        assert!(out.starts_with("error:"), "{out}");
    }
}

#[test]
fn mod_add_refuses_a_plain_http_url() {
    let harness = Harness::new();
    let dir = instance(&harness, "one");
    InstanceFolder::create(&dir, "1.26.60.05").with_amml(Some("26.60.1"));
    let (code, out) = run(&harness, &["mod", "add", "one", "http://127.0.0.1:9/x.zip"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("https"), "{out}");
}

fn local_instance(harness: &Harness, name: &str) {
    let dir = instance(harness, name);
    InstanceFolder::create(&dir, "1.26.62.01").with_amml(Some("26.60.5+local")).with_amml_api("2");
}

#[test]
fn instance_list_marks_local_build() {
    let harness = Harness::new();
    local_instance(&harness, "loc");

    let (code, out) = run(&harness, &["instance", "list"]);

    assert_eq!(code, 0, "{out}");
    assert!(out.contains("loc  1.26.62.01  amml 26.60.5+local (local build)  mods 0"), "{out}");
}

#[test]
fn instance_show_marks_local_build() {
    let harness = Harness::new();
    local_instance(&harness, "loc");

    let (code, out) = run(&harness, &["instance", "show", "loc"]);

    assert_eq!(code, 0, "{out}");
    assert!(out.contains("  amml: 26.60.5+local (local build)"), "{out}");
}

#[test]
fn adopt_candidates_mark_local_build() {
    let harness = Harness::new();
    local_instance(&harness, "loc");

    let (code, out) = run(&harness, &["instance", "adopt"]);

    assert_eq!(code, 0, "{out}");
    assert!(out.contains("loc  1.26.62.01  amml 26.60.5+local (local build)  mods 0"), "{out}");
}

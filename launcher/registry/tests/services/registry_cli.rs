use std::fs;
use std::path::{Path, PathBuf};

use amml_registry::{CliOutput, RegistryCli, RegistryRoot};
use tempfile::TempDir;

use crate::support::{DirReleaseSource, Samples, ZipFixture};

const ALLOW: &str = "amml { repo = \"alaydriem/amml\" }\n\nmod \"alaydriem\" \"zoom\" {\n  repo    = \"alaydriem/amml-zoom\"\n  default = true\n}\n";
const REPO: &str = "alaydriem/amml-zoom";

struct Rig {
    _dir: TempDir,
    releases: PathBuf,
    tree: PathBuf,
    allow: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let releases = dir.path().join("releases");
        let tree = dir.path().join("tree");
        let allow = dir.path().join("mods.hcl");
        fs::create_dir_all(&tree).unwrap();
        fs::write(&allow, ALLOW).unwrap();
        Self { _dir: dir, releases, tree, allow }
    }

    fn source(&self) -> DirReleaseSource {
        DirReleaseSource::new(self.releases.clone())
    }

    fn publish_zoom(&self, version: &str, extra_file: &[u8]) {
        let manifest = ZipFixture::manifest("zoom", Some(version), None);
        let zip = ZipFixture::new()
            .file("@alaydriem/zoom/amml.hcl", manifest.as_bytes())
            .file("@alaydriem/zoom/alaydriem_zoom.dll", extra_file)
            .build();
        let asset = format!("zoom-{version}-windows-x64.zip");
        self.source().put_release(REPO, version, &asset, &zip);
    }

    fn publish_zoom_with_icon(&self, version: &str, icon: Option<&[u8]>) {
        let manifest = ZipFixture::manifest("zoom", Some(version), icon.map(|_| "icon.png"));
        let mut zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes());
        if let Some(bytes) = icon {
            zip = zip.file("@alaydriem/zoom/icon.png", bytes);
        }
        let asset = format!("zoom-{version}-windows-x64.zip");
        self.source().put_release(REPO, version, &asset, &zip.build());
    }

    fn run(&self, args: &[&str]) -> CliOutput {
        let mut full = vec!["amml-registry".to_string()];
        full.extend(args.iter().map(|arg| arg.to_string()));
        RegistryCli::new(self.source()).run(&full)
    }

    fn add_mod(&self, version: &str) -> CliOutput {
        self.run(&[
            "add-mod",
            "--tree",
            self.tree.to_str().unwrap(),
            "--allow",
            self.allow.to_str().unwrap(),
            "--repo",
            REPO,
            "--namespace",
            "alaydriem",
            "--name",
            "zoom",
            "--version",
            version,
            "--now",
            Samples::NOW,
        ])
    }

    fn snapshot(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        Self::collect(&self.tree, &mut files);
        files.sort();
        files
    }

    fn collect(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Self::collect(&path, out);
            } else {
                let bytes = fs::read(&path).unwrap();
                out.push((path, bytes));
            }
        }
    }
}

#[test]
fn add_mod_writes_the_document_and_the_derived_files_and_returns_0() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    let output = rig.add_mod("26.60.1");
    assert_eq!(output.code, 0, "{}", output.stderr);

    let document = fs::read_to_string(rig.tree.join("mods/@alaydriem/zoom.json")).unwrap();
    assert!(document.contains("\"26.60.1\""));
    assert!(document.contains("\"title\": \"Zoom\""));
    let root: RegistryRoot = serde_json::from_slice(&fs::read(rig.tree.join("registry.json")).unwrap()).unwrap();
    assert_eq!(root.mod_count, 1);
    assert_eq!(root.published, Samples::NOW);
    assert!(rig.tree.join("names.json").is_file());
    let page = fs::read_to_string(rig.tree.join("index/api-2/0001.json")).unwrap();
    assert!(page.contains("@alaydriem/zoom"));
}

#[test]
fn re_run_returns_10_and_leaves_every_file() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    assert_eq!(rig.add_mod("26.60.1").code, 0);
    let before = rig.snapshot();
    let second = rig.add_mod("26.60.1");
    assert_eq!(second.code, 10, "{}", second.stderr);
    assert_eq!(rig.snapshot(), before);
}

#[test]
fn conflict_returns_1_names_the_version_and_leaves_every_file() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    assert_eq!(rig.add_mod("26.60.1").code, 0);
    let before = rig.snapshot();
    rig.publish_zoom("26.60.1", b"MZ-different-build");
    let output = rig.add_mod("26.60.1");
    assert_eq!(output.code, 1);
    assert!(output.stderr.starts_with("error: "), "{}", output.stderr);
    assert!(output.stderr.contains("26.60.1"), "{}", output.stderr);
    assert_eq!(rig.snapshot(), before);
}

#[test]
fn a_missing_release_returns_1_and_writes_nothing() {
    let rig = Rig::new();
    let output = rig.add_mod("26.60.9");
    assert_eq!(output.code, 1);
    assert!(output.stderr.contains("not found"), "{}", output.stderr);
    assert!(rig.snapshot().is_empty());
}

#[test]
fn a_second_mod_version_adds_to_the_same_document() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    rig.publish_zoom("26.60.2", b"MZ2");
    assert_eq!(rig.add_mod("26.60.1").code, 0);
    assert_eq!(rig.add_mod("26.60.2").code, 0);
    let document = fs::read_to_string(rig.tree.join("mods/@alaydriem/zoom.json")).unwrap();
    assert!(document.find("26.60.2").unwrap() < document.find("26.60.1").unwrap());
}

#[test]
fn add_amml_keeps_the_mod_documents_and_lists_the_release() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    assert_eq!(rig.add_mod("26.60.1").code, 0);

    let zip = ZipFixture::new()
        .file("amml.dll", b"MZ1")
        .file("amml-patch.exe", b"MZ2")
        .file("amml/recipes/a.hcl", b"x")
        .file("amml/amml-version.txt", b"26.60.1")
        .build();
    rig.source().put_release("alaydriem/amml", "26.60.1", "amml-26.60.1-windows-x64.zip", &zip);
    let args = [
        "add-amml",
        "--tree",
        rig.tree.to_str().unwrap(),
        "--allow",
        rig.allow.to_str().unwrap(),
        "--repo",
        "alaydriem/amml",
        "--version",
        "26.60.1",
        "--api-version",
        "2",
        "--game-versions",
        "1.26.60.5,1.26.60.6",
        "--now",
        "2026-11-03T00:00:00Z",
    ];
    let output = rig.run(&args);
    assert_eq!(output.code, 0, "{}", output.stderr);
    let root: RegistryRoot = serde_json::from_slice(&fs::read(rig.tree.join("registry.json")).unwrap()).unwrap();
    assert_eq!(root.amml.versions[0].game_versions, ["1.26.60.5", "1.26.60.6"]);
    assert_eq!(root.mod_count, 1);
    assert_eq!(rig.run(&args).code, 10);
}

#[test]
fn build_tree_follows_the_allow_list() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    assert_eq!(rig.add_mod("26.60.1").code, 0);
    fs::write(&rig.allow, ALLOW.replace("default = true", "core    = true")).unwrap();
    let output = rig.run(&["build-tree", "--tree", rig.tree.to_str().unwrap(), "--allow", rig.allow.to_str().unwrap()]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    let page = fs::read_to_string(rig.tree.join("index/api-2/0001.json")).unwrap();
    assert!(page.contains("\"core\":true"));

    let before = rig.snapshot();
    rig.run(&["build-tree", "--tree", rig.tree.to_str().unwrap(), "--allow", rig.allow.to_str().unwrap()]);
    assert_eq!(rig.snapshot(), before);
}

#[test]
fn next_tag_prints_the_tag() {
    let rig = Rig::new();
    let output = rig.run(&["next-tag", "--date", "2026-11-02", "--existing", "2026.11.02-1", "2026.11.01-3"]);
    assert_eq!(output.code, 0);
    assert_eq!(output.stdout, "2026.11.02-2\n");
    let first = rig.run(&["next-tag", "--date", "2026-11-02"]);
    assert_eq!(first.stdout, "2026.11.02-1\n");
}

#[test]
fn lint_manifest_accepts_a_good_manifest_and_names_the_bad_field() {
    let rig = Rig::new();
    let path = rig._dir.path().join("amml.hcl");
    fs::write(&path, ZipFixture::manifest("zoom", Some("26.60.1"), None)).unwrap();
    let ok = rig.run(&["lint-manifest", path.to_str().unwrap(), "--version", "26.60.1"]);
    assert_eq!(ok.code, 0, "{}", ok.stderr);

    let wrong = rig.run(&["lint-manifest", path.to_str().unwrap(), "--version", "26.60.2"]);
    assert_eq!(wrong.code, 1);
    assert!(wrong.stderr.contains("version"));

    fs::write(&path, ZipFixture::manifest("zoom", Some("26.60.1"), None).replace("Hold a key to zoom.", "")).unwrap();
    let bad = rig.run(&["lint-manifest", path.to_str().unwrap()]);
    assert_eq!(bad.code, 1);
    assert!(bad.stderr.contains("summary"), "{}", bad.stderr);
}

#[test]
fn lint_manifest_refuses_amml_version_below_the_floor() {
    let rig = Rig::new();
    let path = rig._dir.path().join("amml.hcl");
    let text = ZipFixture::manifest("zoom", Some("26.60.1"), None)
        .replace("api_version = 2\n", "api_version = 2\namml_version = \"26.61.0\"\n");
    fs::write(&path, text).unwrap();
    let low = rig.run(&["lint-manifest", path.to_str().unwrap()]);
    assert_eq!(low.code, 1);
    assert!(low.stderr.contains("26.62.0"), "{}", low.stderr);
}

#[test]
fn lint_manifest_checks_the_icon_file_next_to_the_manifest() {
    let rig = Rig::new();
    let path = rig._dir.path().join("amml.hcl");
    fs::write(&path, ZipFixture::manifest("zoom", Some("26.60.1"), Some("icon.png"))).unwrap();
    let missing = rig.run(&["lint-manifest", path.to_str().unwrap()]);
    assert!(missing.stderr.contains("icon"), "{}", missing.stderr);
    fs::write(rig._dir.path().join("icon.png"), ZipFixture::png(128, 128, 1000)).unwrap();
    assert_eq!(rig.run(&["lint-manifest", path.to_str().unwrap()]).code, 0);
    fs::write(rig._dir.path().join("icon.png"), ZipFixture::png(32, 32, 1000)).unwrap();
    assert_eq!(rig.run(&["lint-manifest", path.to_str().unwrap()]).code, 1);
}

#[test]
fn unknown_arguments_return_1() {
    let rig = Rig::new();
    let output = rig.run(&["add-mod", "--nope"]);
    assert_eq!(output.code, 1);
    assert!(output.stderr.starts_with("error: "));
}

#[test]
fn a_mod_that_is_not_allowed_fails_before_any_download() {
    let rig = Rig::new();
    let (tree, allow) = (rig.tree.to_str().unwrap(), rig.allow.to_str().unwrap());
    let output = rig.run(&[
        "add-mod",
        "--tree",
        tree,
        "--allow",
        allow,
        "--repo",
        REPO,
        "--namespace",
        "alaydriem",
        "--name",
        "toggles",
        "--version",
        "26.60.1",
    ]);
    assert_eq!(output.code, 1);
    assert!(output.stderr.contains("@alaydriem/toggles is not in the allow list"), "{}", output.stderr);
    let wrong_repo = rig.run(&[
        "add-mod",
        "--tree",
        tree,
        "--allow",
        allow,
        "--repo",
        "x/y",
        "--namespace",
        "alaydriem",
        "--name",
        "zoom",
        "--version",
        "26.60.1",
    ]);
    assert!(wrong_repo.stderr.contains("repo mismatch"), "{}", wrong_repo.stderr);
    assert!(rig.snapshot().is_empty());
}

#[test]
fn add_mod_prints_the_checksum_and_the_asset_url_of_the_allow_list_repo() {
    let rig = Rig::new();
    rig.publish_zoom("26.60.1", b"MZ");
    let (tree, allow) = (rig.tree.to_str().unwrap(), rig.allow.to_str().unwrap());
    let output = rig.run(&[
        "add-mod",
        "--tree",
        tree,
        "--allow",
        allow,
        "--repo",
        "Alaydriem/AMML-Zoom",
        "--namespace",
        "alaydriem",
        "--name",
        "zoom",
        "--version",
        "26.60.1",
    ]);
    assert_eq!(output.code, 0, "{}", output.stderr);
    assert!(output.stdout.contains("sha256 "), "{}", output.stdout);
    assert!(
        output.stdout.contains("asset https://github.com/alaydriem/amml-zoom/releases/download/26.60.1/"),
        "{}",
        output.stdout
    );
}

#[test]
fn a_backfill_keeps_the_newest_icon_and_a_newest_release_without_icon_removes_it() {
    let rig = Rig::new();
    let newest = ZipFixture::png(128, 128, 2000);
    let older = ZipFixture::png(128, 128, 3000);
    rig.publish_zoom_with_icon("26.60.2", Some(&newest));
    rig.publish_zoom_with_icon("26.60.1", Some(&older));
    assert_eq!(rig.add_mod("26.60.2").code, 0);
    assert_eq!(rig.add_mod("26.60.1").code, 0);
    let icon = rig.tree.join("icons/@alaydriem/zoom.png");
    assert_eq!(fs::read(&icon).unwrap(), newest);

    rig.publish_zoom_with_icon("26.60.3", None);
    assert_eq!(rig.add_mod("26.60.3").code, 0);
    assert!(!icon.exists());
}

#[test]
fn lint_manifest_checks_namespace_name_and_api_version() {
    let rig = Rig::new();
    let path = rig._dir.path().join("amml.hcl");
    fs::write(&path, ZipFixture::manifest("zoom", Some("26.60.1"), None)).unwrap();
    let p = path.to_str().unwrap();
    assert_eq!(rig.run(&["lint-manifest", p, "--namespace", "alaydriem", "--name", "zoom"]).code, 0);
    let wrong_name = rig.run(&["lint-manifest", p, "--name", "other"]);
    assert_eq!(wrong_name.code, 1);
    assert!(wrong_name.stderr.contains("name"), "{}", wrong_name.stderr);
    assert_eq!(rig.run(&["lint-manifest", p, "--namespace", "other"]).code, 1);

    fs::write(&path, ZipFixture::manifest("zoom", Some("26.60.1"), None).replace("api_version = 2\n", "")).unwrap();
    let no_api = rig.run(&["lint-manifest", p]);
    assert_eq!(no_api.code, 1);
    assert!(no_api.stderr.contains("api_version"), "{}", no_api.stderr);
}

#[test]
fn add_mod_refuses_min_amml_above_newest() {
    let rig = Rig::new();
    let manifest = ZipFixture::manifest("zoom", Some("26.62.1"), None).replacen(
        "api_version = 2\n",
        "api_version = 2\namml_version = \"26.62.5\"\n",
        1,
    );
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).build();
    rig.source().put_release(REPO, "26.62.1", "zoom-26.62.1-windows-x64.zip", &zip);
    let before = rig.snapshot();
    let output = rig.add_mod("26.62.1");
    assert_ne!(output.code, 0);
    assert_ne!(output.code, 10);
    assert!(output.stderr.contains("26.62.5"), "{}", output.stderr);
    assert_eq!(rig.snapshot(), before);
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use amml_proof_fixture::{DirServer, DummyMod, TreeMaker};
use amml_registry::{CliOutput, HttpReleaseSource, RegistryCli};
use serde_json::Value;
use tempfile::TempDir;

const GATED_ALLOW: &str = "\nmod \"proof\" \"gated\" {\n  repo    = \"proof/amml-gated\"\n  default = true\n}\n";

// a frozen copy of the live registry, copied to a temp folder
struct RealRegistry {
    _dir: TempDir,
    tree: PathBuf,
}

impl RealRegistry {
    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/amml-registry-9239b8b")
    }

    fn copy() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        Self::copy_dir(&Self::fixture(), &tree);
        Self { _dir: dir, tree }
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let path = entry.unwrap().path();
            let target = to.join(path.file_name().unwrap());
            if path.is_dir() {
                Self::copy_dir(&path, &target);
            } else {
                fs::copy(&path, &target).unwrap();
            }
        }
    }

    // every file under a folder, keyed by its relative path with forward slashes
    fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        Self::collect(root, root, &mut out);
        out
    }

    fn collect(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Self::collect(root, &path, out);
            } else {
                let relative =
                    path.strip_prefix(root).unwrap().to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
                out.insert(relative, fs::read(&path).unwrap());
            }
        }
    }

    fn changed(before: &BTreeMap<String, Vec<u8>>, after: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
        let mut paths: Vec<String> = before.keys().chain(after.keys()).cloned().collect();
        paths.sort();
        paths.dedup();
        paths.into_iter().filter(|path| before.get(path) != after.get(path)).collect()
    }
}

#[test]
fn rebuild_is_byte_identical() {
    let copy = RealRegistry::copy();
    let before = RealRegistry::files(&RealRegistry::fixture());
    assert!(before.contains_key("registry.json"), "the fixture lists registry.json");

    let agent: ureq::Agent = ureq::Agent::config_builder().build().into();
    let source = HttpReleaseSource::new(agent, "http://127.0.0.1:9/gh".to_string(), 0, std::time::Duration::ZERO);
    let tree = copy.tree.to_str().unwrap();
    let allow = copy.tree.join("mods.hcl");
    let args = ["amml-registry", "build-tree", "--tree", tree, "--allow", allow.to_str().unwrap()];
    let output = RegistryCli::new(source).run(&args.map(String::from));
    assert!(output.code == 0 || output.code == CliOutput::UNCHANGED, "build-tree: {}", output.stderr);

    let after = RealRegistry::files(&copy.tree);
    assert_eq!(RealRegistry::changed(&before, &after), Vec::<String>::new(), "real registry / rebuild");
}

#[test]
fn adding_a_gated_release_changes_only_its_files() {
    let copy = RealRegistry::copy();
    let before = RealRegistry::files(&copy.tree);

    let work = tempfile::tempdir().unwrap();
    let serve = work.path().join("serve");
    fs::create_dir_all(&serve).unwrap();
    let server = DirServer::start(&serve);
    let allow = copy.tree.join("mods.hcl");
    let maker = TreeMaker::new(&server.base(), &serve, &copy.tree, &allow);
    let mut text = String::from_utf8(before["mods.hcl"].clone()).unwrap();
    text.push_str(GATED_ALLOW);
    fs::write(&allow, text).unwrap();

    let gated = DummyMod::MATRIX[3];
    assert_eq!((gated.name, gated.version, gated.amml_version), ("gated", "2.0.0", Some("26.60.6")));
    TreeMaker::expect_changed("real registry / gated 2.0.0 / add-mod", maker.add_mod(&gated));

    let after = RealRegistry::files(&copy.tree);
    assert_eq!(
        RealRegistry::changed(&before, &after),
        ["index/api-2/0001.json", "mods.hcl", "mods/@proof/gated.json", "names.json", "registry.json"],
        "real registry / gated 2.0.0 / add-mod"
    );

    let parse =
        |files: &BTreeMap<String, Vec<u8>>, path: &str| -> Value { serde_json::from_slice(&files[path]).unwrap() };

    // the shared index page: every card but the new one is unchanged, and the new one carries the floor
    let page_before = parse(&before, "index/api-2/0001.json");
    let mut page_after = parse(&after, "index/api-2/0001.json");
    let cards = page_after["cards"].as_array_mut().unwrap();
    let at = cards.iter().position(|card| card["id"] == "@proof/gated").expect("the gated card is on the page");
    let gated_card = cards.remove(at);
    assert_eq!(gated_card["min_amml"], "26.60.6", "real registry / gated card / min_amml");
    assert_eq!(page_after, page_before, "real registry / index page / every other card");

    // registry.json: only the date, the counts and the names file digest move
    let mut root_before = parse(&before, "registry.json");
    let mut root_after = parse(&after, "registry.json");
    let mods = root_before["mod_count"].as_u64().unwrap();
    assert_eq!(root_after["mod_count"].as_u64().unwrap(), mods + 1, "real registry / mod_count");
    let listed = root_before["index"]["apis"]["2"]["mods"].as_u64().unwrap();
    assert_eq!(root_after["index"]["apis"]["2"]["mods"].as_u64().unwrap(), listed + 1, "real registry / index mods");
    assert_eq!(root_after["index"]["apis"]["2"]["pages"], root_before["index"]["apis"]["2"]["pages"]);
    assert_eq!(root_after["names"]["path"], root_before["names"]["path"]);
    for root in [&mut root_before, &mut root_after] {
        let map = root.as_object_mut().unwrap();
        for key in ["published", "mod_count", "index", "names"] {
            map.remove(key);
        }
    }
    assert_eq!(root_after, root_before, "real registry / registry.json / everything else");

    let document: Value = serde_json::from_slice(&after["mods/@proof/gated.json"]).unwrap();
    let release = &document["versions"][0];
    assert_eq!(release["version"], "2.0.0");
    assert_eq!(release["min_amml"], "26.60.6", "real registry / gated 2.0.0 / min_amml");
    let asset = release["asset"].as_str().unwrap();
    assert!(asset.starts_with(&server.base()), "asset {asset}");
}

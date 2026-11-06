use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use amml_registry::{CliOutput, HttpReleaseSource, RegistryCli};

use crate::services::ZipMaker;
use crate::structs::DummyMod;

/// Publishes dummy releases under the served folder and drives the real registry CLI against them.
pub struct TreeMaker {
    cli: RegistryCli<HttpReleaseSource>,
    serve: PathBuf,
    tree: PathBuf,
    allow: PathBuf,
}

impl TreeMaker {
    pub const NOW: &'static str = "2026-11-05T10:00:00Z";
    pub const AMML_REPO: &'static str = "proof/amml";
    pub const ALLOW: &'static str = "amml {\n  repo = \"proof/amml\"\n}\n\n\
        mod \"proof\" \"plain\" {\n  repo    = \"proof/amml-plain\"\n  default = true\n}\n\n\
        mod \"proof\" \"gated\" {\n  repo    = \"proof/amml-gated\"\n  default = true\n}\n\n\
        mod \"proof\" \"onlygated\" {\n  repo    = \"proof/amml-onlygated\"\n  default = true\n}\n";

    /// `base` is the local server address; releases are read from `{base}/gh`.
    pub fn new(base: &str, serve: &Path, tree: &Path, allow: &Path) -> Self {
        let agent: ureq::Agent =
            ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(30))).build().into();
        let source = HttpReleaseSource::new(agent, format!("{base}/gh"), 0, Duration::ZERO);
        fs::create_dir_all(tree).unwrap();
        fs::write(allow, Self::ALLOW).unwrap();
        Self {
            cli: RegistryCli::new(source),
            serve: serve.to_path_buf(),
            tree: tree.to_path_buf(),
            allow: allow.to_path_buf(),
        }
    }

    /// The folder of one release's assets inside the served folder.
    pub fn release_dir(&self, repo: &str, tag: &str) -> PathBuf {
        self.serve.join("gh").join(repo).join("releases").join("download").join(tag)
    }

    pub fn publish_amml(&self, version: &str) {
        let zip = ZipMaker::amml_zip(version);
        self.put(Self::AMML_REPO, version, &ZipMaker::amml_asset(version), &zip);
    }

    pub fn publish_mod(&self, dummy: &DummyMod) -> PathBuf {
        let zip = ZipMaker::mod_zip(dummy);
        self.put(&dummy.repo(), dummy.version, &dummy.asset(), &zip)
    }

    /// Publishes the AMML release and lists it.
    pub fn add_amml(&self, version: &str, game: &str) -> CliOutput {
        self.publish_amml(version);
        self.run(
            "add-amml",
            &["--repo", Self::AMML_REPO, "--version", version, "--api-version", "2", "--game-versions", game],
        )
    }

    /// Publishes the dummy release and lists it.
    pub fn add_mod(&self, dummy: &DummyMod) -> CliOutput {
        self.publish_mod(dummy);
        self.run(
            "add-mod",
            &[
                "--repo",
                &dummy.repo(),
                "--namespace",
                DummyMod::NAMESPACE,
                "--name",
                dummy.name,
                "--version",
                dummy.version,
            ],
        )
    }

    /// Panics with the CLI stderr unless the command changed the tree.
    pub fn expect_changed(what: &str, output: CliOutput) {
        assert_eq!(output.code, CliOutput::CHANGED, "{what}: {}", output.stderr);
    }

    fn put(&self, repo: &str, tag: &str, asset: &str, zip: &[u8]) -> PathBuf {
        let dir = self.release_dir(repo, tag);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(asset), zip).unwrap();
        fs::write(dir.join("SHA256SUMS.txt"), ZipMaker::sums(asset, zip)).unwrap();
        dir.join(asset)
    }

    fn run(&self, command: &str, args: &[&str]) -> CliOutput {
        let mut full = vec!["amml-registry".to_string(), command.to_string()];
        full.extend(["--tree", self.tree.to_str().unwrap(), "--allow", self.allow.to_str().unwrap()].map(String::from));
        full.extend(args.iter().map(|arg| arg.to_string()));
        full.extend(["--now".to_string(), Self::NOW.to_string()]);
        self.cli.run(&full)
    }
}

use std::fs;
use std::path::{Path, PathBuf};

use amml_registry::CliOutput;

use crate::services::{DirServer, TreeMaker};
use crate::structs::DummyMod;

/// A registry tree built by the real CLI from dummy releases, served over local HTTP.
pub struct ProofTree {
    pub server: DirServer,
    pub tree: PathBuf,
    pub registry_base: String,
    work: PathBuf,
    maker: TreeMaker,
}

impl ProofTree {
    pub const OLD_AMML: &'static str = "26.60.5";
    pub const OLD_GAME: &'static str = "1.26.60.5";
    pub const NEW_AMML: &'static str = "26.62.0";
    pub const NEW_GAME: &'static str = "1.26.62.1";

    /// Lists OLD and NEW AMML, then the five matrix mods.
    pub fn build(work: &Path) -> ProofTree {
        let proof = Self::start(work);
        Self::list_amml(&proof, Self::OLD_AMML, Self::OLD_GAME);
        Self::list_amml(&proof, Self::NEW_AMML, Self::NEW_GAME);
        for dummy in DummyMod::MATRIX {
            let what = format!("add-mod {} {}", dummy.id(), dummy.version);
            TreeMaker::expect_changed(&what, proof.maker.add_mod(&dummy));
        }
        proof
    }

    /// Lists OLD AMML only.
    pub fn build_old_only(work: &Path) -> ProofTree {
        let proof = Self::start(work);
        Self::list_amml(&proof, Self::OLD_AMML, Self::OLD_GAME);
        proof
    }

    /// The release zip of a matrix mod, as served.
    pub fn mod_zip(&self, name: &str, version: &str) -> PathBuf {
        let dummy = DummyMod::MATRIX
            .iter()
            .find(|dummy| dummy.name == name && dummy.version == version)
            .unwrap_or_else(|| panic!("{name} {version} is not in the matrix"));
        self.maker.release_dir(&dummy.repo(), version).join(dummy.asset())
    }

    /// Publishes a dummy release and runs `add-mod` for it, without checking the result.
    pub fn try_add_mod(&self, dummy: &DummyMod) -> CliOutput {
        self.maker.add_mod(dummy)
    }

    /// Every file of the tree with its bytes, in path order.
    pub fn snapshot(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        Self::collect(&self.tree, &mut files);
        files.sort();
        files
    }

    pub fn work(&self) -> &Path {
        &self.work
    }

    fn start(work: &Path) -> ProofTree {
        let serve = work.join("serve");
        let tree = serve.join("registry");
        fs::create_dir_all(&serve).unwrap();
        let server = DirServer::start(&serve);
        let registry_base = format!("{}/registry", server.base());
        let maker = TreeMaker::new(&server.base(), &serve, &tree, &work.join("mods.hcl"));
        ProofTree { server, tree, registry_base, work: work.to_path_buf(), maker }
    }

    fn list_amml(proof: &ProofTree, version: &str, game: &str) {
        TreeMaker::expect_changed(&format!("add-amml {version}"), proof.maker.add_amml(version, game));
    }

    fn collect(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Self::collect(&path, out);
            } else {
                out.push((path.clone(), fs::read(&path).unwrap()));
            }
        }
    }
}

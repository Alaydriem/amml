use std::io::{Cursor, Write};

use amml_registry::HashText;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::structs::DummyMod;

/// Builds the dummy release zips, in the layout the registry verifier accepts.
pub struct ZipMaker;

impl ZipMaker {
    pub fn amml_zip(version: &str) -> Vec<u8> {
        Self::build(&[
            ("amml.dll", b"MZ-dummy-loader".to_vec()),
            ("amml-patch.exe", b"MZ-dummy-patcher".to_vec()),
            ("amml/recipes/dummy.hcl", b"# dummy recipe\n".to_vec()),
            ("amml/amml-version.txt", format!("{version}\n").into_bytes()),
        ])
    }

    pub fn mod_zip(dummy: &DummyMod) -> Vec<u8> {
        let folder = format!("@{}/{}", DummyMod::NAMESPACE, dummy.name);
        Self::build(&[
            (&format!("{folder}/amml.hcl"), Self::manifest(dummy).into_bytes()),
            (&format!("{folder}/proof_{}.dll", dummy.name), b"MZ-dummy-mod".to_vec()),
        ])
    }

    pub fn amml_asset(version: &str) -> String {
        format!("amml-{version}-windows-x64.zip")
    }

    /// The `SHA256SUMS.txt` line for one asset.
    pub fn sums(asset: &str, zip: &[u8]) -> String {
        format!("{}  {asset}\n", HashText::sha256(zip))
    }

    pub fn manifest(dummy: &DummyMod) -> String {
        let minimum = dummy.amml_version.map(|v| format!("amml_version = \"{v}\"\n")).unwrap_or_default();
        format!(
            "namespace   = \"{ns}\"\nname        = \"{name}\"\nentry       = \"proof_{name}.dll\"\napi_version = 2\nversion     = \"{version}\"\n{minimum}\n\
             listing {{\n  title       = \"Proof {name}\"\n  summary     = \"A dummy mod for the proof tests.\"\n  authors     = [\"Charles R. Portwood II\"]\n  license     = \"BSD-3-Clause\"\n  source      = \"https://github.com/proof/amml-{name}\"\n  tags        = [\"proof\"]\n}}\n",
            ns = DummyMod::NAMESPACE,
            name = dummy.name,
            version = dummy.version,
        )
    }

    fn build(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (path, bytes) in entries {
            writer.start_file(*path, SimpleFileOptions::default()).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
}

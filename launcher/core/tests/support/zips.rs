use std::io::Write;
use std::path::{Path, PathBuf};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

// Mod and AMML release zips built in a temp folder.
pub struct Zips;

impl Zips {
    // an `amml.hcl` for a mod, with a listing block
    pub fn manifest(namespace: &str, name: &str, version: Option<&str>, api: u32) -> String {
        let version_line = version.map(|v| format!("version     = \"{v}\"\n")).unwrap_or_default();
        format!(
            "namespace   = \"{namespace}\"\nname        = \"{name}\"\nentry       = \"{name}.dll\"\napi_version = {api}\n{version_line}\n\
             listing {{\n  title       = \"{title}\"\n  summary     = \"{title} summary\"\n  authors     = [\"Charles R. Portwood II\"]\n  license     = \"BSD-3-Clause\"\n  source      = \"https://github.com/alaydriem/amml\"\n  tags        = [\"camera\"]\n}}\n",
            title = name.to_uppercase()
        )
    }

    // any entries at all, by name; a name ending in `/` is a folder
    pub fn raw(path: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = ZipWriter::new(file);
        for (name, bytes) in entries {
            if name.ends_with('/') {
                writer.add_directory(*name, SimpleFileOptions::default()).unwrap();
            } else {
                writer.start_file(*name, SimpleFileOptions::default()).unwrap();
                writer.write_all(bytes).unwrap();
            }
        }
        writer.finish().unwrap();
        path.to_path_buf()
    }

    // a mod zip: `@namespace/name/amml.hcl`, `name.dll` and the extra files below the mod folder
    pub fn mod_zip(
        path: &Path,
        namespace: &str,
        name: &str,
        version: &str,
        api: u32,
        extra: &[(&str, &[u8])],
    ) -> PathBuf {
        let manifest = Self::manifest(namespace, name, Some(version), api);
        let prefix = format!("@{namespace}/{name}");
        let mut entries: Vec<(String, Vec<u8>)> = vec![
            (format!("{prefix}/amml.hcl"), manifest.into_bytes()),
            (format!("{prefix}/{name}.dll"), format!("dll {version}").into_bytes()),
        ];
        for (relative, bytes) in extra {
            entries.push((format!("{prefix}/{relative}"), bytes.to_vec()));
        }
        let borrowed: Vec<(&str, &[u8])> = entries.iter().map(|(n, b)| (n.as_str(), b.as_slice())).collect();
        Self::raw(path, &borrowed)
    }

    // an AMML zip; the patcher in it is only a marker file because the tests use a fake patcher
    pub fn amml_zip(path: &Path, version: &str, with_patcher: bool) -> PathBuf {
        let dll = format!("loader {version}");
        let mut entries: Vec<(&str, &[u8])> = vec![
            ("amml.dll", dll.as_bytes()),
            ("amml/recipes/a.hcl", b"recipe a"),
            ("amml/recipes/sub/b.hcl", b"recipe b"),
            ("amml/amml-version.txt", version.as_bytes()),
        ];
        if with_patcher {
            entries.push(("amml-patch.exe", b"patcher"));
        }
        Self::raw(path, &entries)
    }
}

use std::io::{Cursor, Write};

use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Builds release zips in memory.
pub struct ZipFixture {
    entries: Vec<(String, Vec<u8>)>,
}

impl ZipFixture {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn file(mut self, path: &str, bytes: &[u8]) -> Self {
        self.entries.push((path.to_string(), bytes.to_vec()));
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        for (path, bytes) in &self.entries {
            writer.start_file(path, SimpleFileOptions::default()).unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    /// A PNG header for an icon of the given size, padded to `total` bytes.
    pub fn png(width: u32, height: u32, total: usize) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.resize(total, 0);
        bytes
    }

    /// An `amml.hcl` with a listing, for `@alaydriem/zoom`.
    pub fn manifest(name: &str, version: Option<&str>, icon: Option<&str>) -> String {
        let version_line = version.map(|v| format!("version     = \"{v}\"\n")).unwrap_or_default();
        let icon_line = icon.map(|i| format!("  icon        = \"{i}\"\n")).unwrap_or_default();
        format!(
            "namespace   = \"alaydriem\"\nname        = \"{name}\"\nentry       = \"alaydriem_zoom.dll\"\napi_version = 2\n{version_line}\n\
             listing {{\n  title       = \"Zoom\"\n  summary     = \"Hold a key to zoom.\"\n  authors     = [\"Charles R. Portwood II\"]\n  license     = \"BSD-3-Clause\"\n  source      = \"https://github.com/alaydriem/amml-zoom\"\n  tags        = [\"camera\"]\n{icon_line}}}\n"
        )
    }
}

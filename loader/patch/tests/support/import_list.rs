use pelite::pe64::imports::Import;
use pelite::pe64::{Pe, PeFile};

/// Every `dll!symbol` import of a PE file, read back with `pelite`.
pub struct ImportList;

impl ImportList {
    pub fn read(bytes: &[u8]) -> Vec<String> {
        let file = PeFile::from_bytes(bytes).unwrap();
        let mut found = Vec::new();
        for descriptor in file.imports().unwrap() {
            let dll = descriptor.dll_name().unwrap().to_string();
            for import in descriptor.int().unwrap() {
                match import.unwrap() {
                    Import::ByName { name, .. } => found.push(format!("{dll}!{name}")),
                    Import::ByOrdinal { ord } => found.push(format!("{dll}!#{ord}")),
                }
            }
        }
        found
    }
}

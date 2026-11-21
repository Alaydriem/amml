use std::path::Path;
use std::path::PathBuf;

use amml_data::{AdapterCatalog, BuildData, EntryId, NeedKind};
use pelite::pe64::imports::Import;
use pelite::pe64::{Pe, PeFile};

use crate::structs::{CompatReport, CompatRow, ImportWarning, ModUses};

/// Checks what the mods declare in `uses` against the entries a build resolved, and lists the forbidden imports of the mod DLLs.
pub struct CompatChecker;

impl CompatChecker {
    /// Win32 functions a mod must not import, each with an optional note. `GetModuleHandleW` and
    /// `GetProcAddress` are not listed: the Rust standard library imports them into every mod DLL.
    pub const FORBIDDEN_IMPORTS: [(&'static str, Option<&'static str>); 11] = [
        ("SetWindowLongPtrW", None),
        ("SetWindowsHookExW", None),
        ("GetAsyncKeyState", None),
        ("GetKeyState", None),
        ("GetCursorInfo", None),
        ("ClipCursor", None),
        ("EnumWindows", None),
        ("CreateDXGIFactory", None),
        ("CreateDXGIFactory1", None),
        ("CreateDXGIFactory2", None),
        ("D3D11On12CreateDevice", None),
    ];

    /// One row per event of each mod, from the entries in `data`, and one warning per forbidden import of each DLL in `dlls`.
    pub fn check(data: &BuildData, mods: &[ModUses], dlls: &[PathBuf]) -> CompatReport {
        let availability = AdapterCatalog::availability(&|name| data.entries.contains_key(&EntryId(name.to_string())));
        let mut report = CompatReport::default();
        for uses in mods {
            for feature in &uses.features {
                let events =
                    feature.events.iter().map(|event| (NeedKind::Event, &event.name, event.version, event.required));
                let areas = feature.areas.iter().map(|area| (NeedKind::Area, &area.name, area.version, area.required));
                for (kind, name, version, required) in events.chain(areas) {
                    let held =
                        availability.iter().find(|row| row.id == name && row.kind == kind && row.version >= version);
                    let (available, shape, missing) = match held {
                        Some(row) => (
                            row.shape.is_some(),
                            row.shape.map(str::to_string),
                            row.missing.iter().map(|name| (*name).to_string()).collect(),
                        ),
                        None => (false, None, vec![Self::too_old(kind, name)]),
                    };
                    report.rows.push(CompatRow {
                        mod_id: uses.mod_id.clone(),
                        feature: feature.name.clone(),
                        kind,
                        name: name.clone(),
                        required,
                        available,
                        shape,
                        missing,
                        loader_too_old: held.is_none(),
                    });
                }
            }
        }
        for dll in dlls {
            report.warnings.extend(Self::imports(dll));
        }
        report
    }

    // Why a row has no catalog need at its version: the loader serves a lower version, or none.
    fn too_old(kind: NeedKind, name: &str) -> String {
        let word = Self::kind_word(kind);
        match AdapterCatalog::served_version(kind, name) {
            Some(served) => format!("loader too old: it serves {word} `{name}` up to version {served}"),
            None => format!("loader too old: it has no {word} `{name}`"),
        }
    }

    fn kind_word(kind: NeedKind) -> &'static str {
        match kind {
            NeedKind::Event => "event",
            NeedKind::Area => "area",
        }
    }

    fn imports(dll: &Path) -> Vec<ImportWarning> {
        let unreadable = |reason: String| {
            vec![ImportWarning {
                dll: dll.to_path_buf(),
                function: "(imports not read)".to_string(),
                note: Some(reason),
            }]
        };
        let bytes = match std::fs::read(dll) {
            Ok(bytes) => bytes,
            Err(error) => return unreadable(error.to_string()),
        };
        let file = match PeFile::from_bytes(&bytes) {
            Ok(file) => file,
            Err(error) => return unreadable(error.to_string()),
        };
        let imports = match file.imports() {
            Ok(imports) => imports,
            Err(error) => return unreadable(error.to_string()),
        };
        let mut found = Vec::new();
        for descriptor in imports {
            let Ok(table) = descriptor.int() else { continue };
            for import in table.flatten() {
                let Import::ByName { name, .. } = import else { continue };
                let Ok(name) = name.to_str() else { continue };
                if let Some((function, note)) = Self::FORBIDDEN_IMPORTS.iter().find(|(function, _)| *function == name) {
                    found.push(ImportWarning {
                        dll: dll.to_path_buf(),
                        function: (*function).to_string(),
                        note: note.map(str::to_string),
                    });
                }
            }
        }
        found
    }
}

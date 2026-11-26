use super::{CompatRow, ImportWarning};

/// The result of a compat check: one row per event a mod uses, and the import warnings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompatReport {
    pub rows: Vec<CompatRow>,
    pub warnings: Vec<ImportWarning>,
}

impl CompatReport {
    /// The ids of the mods with a required row that is not available, in report order and without repeats.
    pub fn blocked_mods(&self) -> Vec<&str> {
        let mut blocked: Vec<&str> = Vec::new();
        for row in self.rows.iter().filter(|row| row.required && !row.available) {
            if !blocked.contains(&row.mod_id.as_str()) {
                blocked.push(&row.mod_id);
            }
        }
        blocked
    }

    /// 0 when every required row of every mod is available, 1 otherwise.
    pub fn exit_code(&self) -> u8 {
        u8::from(!self.blocked_mods().is_empty())
    }

    /// A table per mod, then the import warnings. A mod with a required row that is not available is marked `will not
    /// enable`; after its rows, one `feature <name>: loader too old` line per feature with a row this loader does not
    /// serve at its version.
    pub fn render(&self) -> String {
        let blocked = self.blocked_mods();
        let mut mods: Vec<&str> = Vec::new();
        for row in &self.rows {
            if !mods.contains(&row.mod_id.as_str()) {
                mods.push(&row.mod_id);
            }
        }
        let mut text = String::new();
        for mod_id in mods {
            text.push_str(mod_id);
            if blocked.contains(&mod_id) {
                text.push_str("  will not enable");
            }
            text.push('\n');
            let mut too_old: Vec<&str> = Vec::new();
            for row in self.rows.iter().filter(|row| row.mod_id == mod_id) {
                let need = if row.required { "required" } else { "optional" };
                let state = match (&row.shape, row.available, row.loader_too_old) {
                    (Some(shape), true, _) => format!("available ({shape})"),
                    (_, _, true) => row.missing.join(", "),
                    _ => format!("missing {}", row.missing.join(", ")),
                };
                text.push_str(&format!("  {:<12} {:<28} {need}  {state}\n", row.feature, row.name));
                if row.loader_too_old && !too_old.contains(&row.feature.as_str()) {
                    too_old.push(&row.feature);
                }
            }
            for feature in too_old {
                text.push_str(&format!("  feature {feature}: loader too old\n"));
            }
        }
        for warning in &self.warnings {
            text.push_str(&format!("warning: {} imports {}", warning.dll.display(), warning.function));
            if let Some(note) = &warning.note {
                text.push_str(&format!(" ({note})"));
            }
            text.push('\n');
        }
        text
    }
}

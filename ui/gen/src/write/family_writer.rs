use super::{OutputTree, VersionOrder};

/// Writes the `mod.rs` files: one per family folder, and the root that declares the families.
///
/// rustfmt sorts the two kinds of line differently, so this writer does too: `mod` lines in plain
/// character order (`k1`, `k10`, `k2`) and `use` lines in version order (`k1`, `k2`, `k10`).
pub struct FamilyWriter;

impl FamilyWriter {
    /// A family `mod.rs`: `mod` lines, then `pub use` lines. Each member is `(module, type)`.
    pub fn render(members: &[(String, String)]) -> String {
        let mut modules: Vec<&(String, String)> = members.iter().collect();
        modules.sort();
        let mut uses = modules.clone();
        uses.sort_by(|(a, _), (b, _)| VersionOrder::compare(a, b));
        let mut out = format!("{}\n", OutputTree::HEADER);
        for (module, _) in &modules {
            out.push_str(&format!("mod {module};\n"));
        }
        out.push('\n');
        for (module, type_name) in &uses {
            out.push_str(&format!("pub use {module}::{type_name};\n"));
        }
        out
    }

    /// The root `mod.rs`: one `pub mod` line per family folder, in character order.
    pub fn render_root(families: &[String]) -> String {
        let mut sorted: Vec<&String> = families.iter().collect();
        sorted.sort();
        sorted.dedup();
        let mut out = format!("{}\n", OutputTree::HEADER);
        for family in sorted {
            out.push_str(&format!("pub mod {family};\n"));
        }
        out
    }
}

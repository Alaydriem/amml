use std::path::{Path, PathBuf};

use amml::HostPaths;
use pelite::pe64::exports::GetProcAddress;
use pelite::pe64::{Pe, PeFile};

/// The `small.dll` fixture used as a game exe, with recipes that find its `amml_probe` export.
pub struct Probe;

impl Probe {
    pub fn dll() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../derive/cli/tests/fixtures/pe/small.dll")
    }

    pub fn rva() -> u32 {
        let bytes = std::fs::read(Self::dll()).unwrap();
        PeFile::from_bytes(&bytes).unwrap().get_export("amml_probe").unwrap().symbol().unwrap()
    }

    /// A standalone game folder under `root`: the fixture copied as `Minecraft.Windows.exe` and
    /// `amml/recipes/probe.hcl` with `Probe.run`, a function matching the prologue of `amml_probe`.
    pub fn standalone_layout(root: &Path) -> HostPaths {
        let paths = HostPaths::standalone(&root.join("Minecraft.Windows.exe"));
        std::fs::copy(Self::dll(), &paths.exe).unwrap();
        let bytes = std::fs::read(Self::dll()).unwrap();
        let prologue = PeFile::from_bytes(&bytes).unwrap().derva_slice::<u8>(Self::rva(), 24).unwrap().to_vec();
        let pattern = prologue.iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ");
        let recipe = format!(
            "entry \"Probe.run\" {{\n  kind = \"function\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"{pattern}\"\n    }}\n  }}\n  check \"function_start\" {{}}\n}}\n"
        );
        std::fs::create_dir_all(&paths.recipes).unwrap();
        std::fs::write(paths.recipes.join("probe.hcl"), recipe).unwrap();
        paths
    }
}

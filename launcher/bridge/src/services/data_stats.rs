use crate::DataFigures;
use std::path::Path;

// Counts what a `Minecraft Bedrock` folder holds, without reading any file.
pub struct DataStats;

impl DataStats {
    const MAX_DEPTH: u32 = 8;
    const WORLDS: &'static str = "minecraftWorlds";
    const PACK_FOLDERS: [&'static str; 2] = ["resource_packs", "behavior_packs"];

    // Worlds are folders with a `level.dat` under any `minecraftWorlds`. Packs are the child folders of any
    // `resource_packs` and `behavior_packs`. Bytes are all files.
    pub fn of(minecraft_bedrock: &Path) -> DataFigures {
        let mut figures = DataFigures::default();
        Self::walk(minecraft_bedrock, "", Self::MAX_DEPTH, &mut figures);
        figures
    }

    fn walk(dir: &Path, name: &str, depth: u32, figures: &mut DataFigures) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                let child_name = entry.file_name().to_string_lossy().to_string();
                if name == Self::WORLDS && entry.path().join("level.dat").is_file() {
                    figures.worlds += 1;
                }
                if name == Self::PACK_FOLDERS[0] {
                    figures.resource_packs += 1;
                }
                if name == Self::PACK_FOLDERS[1] {
                    figures.behavior_packs += 1;
                }
                if depth > 1 {
                    Self::walk(&entry.path(), &child_name, depth - 1, figures);
                }
            } else if let Ok(meta) = entry.metadata() {
                figures.bytes += meta.len();
            }
        }
    }
}

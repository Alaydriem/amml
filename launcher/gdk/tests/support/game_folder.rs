use super::PeBytes;
use std::path::Path;

pub struct GameFolder;

impl GameFolder {
    pub const FILES: u64 = 5;

    // a synthetic game folder of `FILES` files: an x64 exe, the game config and three data files
    pub fn create(dir: &Path) {
        std::fs::create_dir_all(dir.join("data").join("definitions")).unwrap();
        std::fs::create_dir_all(dir.join("libs")).unwrap();
        std::fs::write(dir.join("Minecraft.Windows.exe"), PeBytes::x64(8192)).unwrap();
        std::fs::write(dir.join("MicrosoftGame.Config"), b"<Game/>").unwrap();
        std::fs::write(dir.join("data").join("definitions").join("a.json"), b"{}").unwrap();
        std::fs::write(dir.join("libs").join("c.dll"), b"dll").unwrap();
        std::fs::write(dir.join("b.txt"), b"b").unwrap();
    }
}

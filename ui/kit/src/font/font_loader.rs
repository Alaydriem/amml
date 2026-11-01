use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

use super::{FontFace, FontSource};

// Nordwand Mono is embedded; the icon font is read from the Windows font folder and never shipped.
// A missing icon file falls back to egui's built-in fonts
pub struct FontLoader;

impl FontLoader {
    pub const TEXT_KEY: &'static str = "nordwand-mono";
    pub const SEMIBOLD_KEY: &'static str = "nordwand-mono-semibold";
    pub const ICONS_KEY: &'static str = "segoe-fluent-icons";
    // Segoe Fluent Icons ships with Windows 11; Windows 10 has Segoe MDL2 Assets, with the same code points
    const ICONS_FILES: [&'static str; 2] = ["SegoeIcons.ttf", "segmdl2.ttf"];
    const DEFAULT_WINDOWS_DIR: &'static str = r"C:\Windows";
    const TEXT: &'static [u8] = include_bytes!("../../assets/fonts/NordwandMono-Medium.ttf");
    const SEMIBOLD: &'static [u8] = include_bytes!("../../assets/fonts/NordwandMono-SemiBold.ttf");

    pub fn install(ctx: &Context, source: FontSource) {
        ctx.set_fonts(Self::definitions(source));
    }

    pub fn definitions(source: FontSource) -> FontDefinitions {
        let mut defs = FontDefinitions::default();
        Self::register(&mut defs, source, &Self::font_dir());
        defs
    }

    /// The Windows font folder, from `WINDIR`.
    pub fn font_dir() -> PathBuf {
        let windows =
            std::env::var_os("WINDIR").map_or_else(|| PathBuf::from(Self::DEFAULT_WINDOWS_DIR), PathBuf::from);
        windows.join("Fonts")
    }

    /// Adds the amml-ui fonts and binds the `FontFace` families and `Monospace`. Other named families stay bound,
    /// so a theme that installs its own fonts can call this on its definitions before `set_fonts`.
    pub fn register(defs: &mut FontDefinitions, source: FontSource, font_dir: &Path) {
        let builtin = FontDefinitions::default();
        let builtin_proportional = builtin.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        let builtin_mono = builtin.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
        let embedded = source == FontSource::Embedded;
        let text = Self::embed(defs, embedded, Self::TEXT_KEY, Self::TEXT);
        let semibold = Self::embed(defs, embedded, Self::SEMIBOLD_KEY, Self::SEMIBOLD);
        let icons = Self::load_icons(defs, embedded, font_dir);

        let text_chain = Self::chain(&[text.clone(), icons.clone()], &builtin_proportional);
        let semibold_chain = Self::chain(&[semibold, text.clone(), icons.clone()], &builtin_proportional);
        let mono_chain = Self::chain(&[text, icons.clone()], &builtin_mono);
        let icon_chain = Self::chain(&[icons], &builtin_proportional);

        defs.families.insert(FontFace::Regular.family(), text_chain);
        defs.families.insert(FontFamily::Monospace, mono_chain);
        defs.families.insert(FontFace::Semibold.family(), semibold_chain);
        defs.families.insert(FontFace::Icons.family(), icon_chain);
    }

    fn embed(defs: &mut FontDefinitions, enabled: bool, key: &str, bytes: &'static [u8]) -> Option<String> {
        if !enabled {
            return None;
        }
        defs.font_data.insert(key.to_owned(), Arc::new(FontData::from_static(bytes)));
        Some(key.to_owned())
    }

    fn load_icons(defs: &mut FontDefinitions, enabled: bool, font_dir: &Path) -> Option<String> {
        if !enabled {
            return None;
        }
        for file in Self::ICONS_FILES {
            if let Ok(bytes) = std::fs::read(font_dir.join(file)) {
                defs.font_data.insert(Self::ICONS_KEY.to_owned(), Arc::new(FontData::from_owned(bytes)));
                return Some(Self::ICONS_KEY.to_owned());
            }
        }
        curia::warn!("icon font not found; using the built-in font", {
            files: Self::ICONS_FILES.join(", "),
            directory: font_dir.display().to_string(),
        });
        None
    }

    fn chain(preferred: &[Option<String>], fallback: &[String]) -> Vec<String> {
        preferred.iter().flatten().cloned().chain(fallback.iter().cloned()).collect()
    }
}

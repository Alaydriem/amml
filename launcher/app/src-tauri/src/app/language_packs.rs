use amml_i18n::negotiate::LocaleNegotiator;
use amml_i18n::pack::LanguagePack;
use std::path::PathBuf;

// Reads the compiled language packs from the bundled resource folder. English has no pack: message ids are the
// English text, so "no pack" is a complete answer.
pub struct LanguagePacks {
    directory: PathBuf,
}

impl LanguagePacks {
    pub fn new(directory: PathBuf) -> LanguagePacks {
        LanguagePacks { directory }
    }

    // the locales with a pack, sorted
    pub fn available(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.directory) else {
            return Vec::new();
        };
        let mut locales: Vec<String> = entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension()?.to_str()? != "json" {
                    return None;
                }
                Some(path.file_stem()?.to_str()?.to_string())
            })
            .collect();
        locales.sort();
        locales
    }

    // the pack that serves a requested locale; `None` for English, for a locale without a pack and for a pack that
    // cannot be read
    pub fn load_for(&self, requested: &str) -> Option<LanguagePack> {
        let locale = LocaleNegotiator::resolve(requested, &self.available())?;
        let path = self.directory.join(format!("{locale}.json"));
        let bytes = std::fs::read(&path).ok()?;
        match serde_json::from_slice::<LanguagePack>(&bytes) {
            Ok(pack) => Some(pack),
            Err(error) => {
                curia::warn!("a language pack is malformed, so English is used", {
                    locale: locale,
                    error: error.to_string(),
                });
                None
            }
        }
    }
}

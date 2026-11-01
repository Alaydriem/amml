use std::borrow::Cow;
use std::path::PathBuf;

/// Where one domain's compiled packs come from.
#[derive(Debug, Clone)]
pub enum PackSource {
    /// Packs built into the binary: `(locale, pack JSON)`.
    Embedded(&'static [(&'static str, &'static [u8])]),
    /// A folder of `<locale>.json` files.
    Directory(PathBuf),
}

impl PackSource {
    /// The locales this source has a pack for, sorted.
    pub fn locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = match self {
            Self::Embedded(packs) => packs.iter().map(|(locale, _)| (*locale).to_owned()).collect(),
            Self::Directory(directory) => match std::fs::read_dir(directory) {
                Ok(entries) => entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.extension()?.to_str()? != "json" {
                            return None;
                        }
                        Some(path.file_stem()?.to_str()?.to_owned())
                    })
                    .collect(),
                Err(_) => Vec::new(),
            },
        };
        locales.sort();
        locales
    }

    /// The raw pack JSON for `locale`, or an error message.
    pub fn read(&self, locale: &str) -> Result<Cow<'static, [u8]>, String> {
        match self {
            Self::Embedded(packs) => packs
                .iter()
                .find(|(known, _)| *known == locale)
                .map(|(_, bytes)| Cow::Borrowed(*bytes))
                .ok_or_else(|| format!("no embedded pack for {locale}")),
            Self::Directory(directory) => std::fs::read(directory.join(format!("{locale}.json")))
                .map(Cow::Owned)
                .map_err(|error| error.to_string()),
        }
    }
}

// Where an installed mod came from, kept in the launcher block of `version.json` as one string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModSource {
    // the base URL of the registry
    Registry(String),
    // an unofficial zip URL; never checked for updates
    Url(String),
    // found on disk, origin unknown
    Local,
}

impl ModSource {
    const URL_PREFIX: &'static str = "url:";
    const LOCAL: &'static str = "local";

    pub fn as_text(&self) -> String {
        match self {
            Self::Registry(base) => base.clone(),
            Self::Url(url) => format!("{}{url}", Self::URL_PREFIX),
            Self::Local => Self::LOCAL.to_string(),
        }
    }

    // text that is neither `local` nor a `url:` form is a registry base URL when it looks like one, else `Local`
    pub fn parse(text: &str) -> ModSource {
        if text == Self::LOCAL {
            return Self::Local;
        }
        if let Some(url) = text.strip_prefix(Self::URL_PREFIX) {
            return Self::Url(url.to_string());
        }
        if text.starts_with("https://") || text.starts_with("http://") {
            return Self::Registry(text.to_string());
        }
        Self::Local
    }
}

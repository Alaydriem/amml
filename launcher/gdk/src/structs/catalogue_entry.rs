use super::{Channel, GameVersion};

#[derive(Debug, Clone)]
pub struct CatalogueEntry {
    pub version: GameVersion,
    pub channel: Channel,
    pub urls: Vec<String>,
}

impl CatalogueEntry {
    // the package file name of the first URL without its extension
    pub fn package_full_name(&self) -> Option<String> {
        let url = self.urls.first()?;
        let path = url.split(['?', '#']).next()?;
        let file = path.rsplit('/').next()?;
        let name = file.strip_suffix(".msixvc")?;
        if name.is_empty() { None } else { Some(name.to_string()) }
    }

    // the file name the package has in the download cache
    pub fn file_name(&self) -> Option<String> {
        self.package_full_name().map(|name| format!("{name}.msixvc"))
    }
}

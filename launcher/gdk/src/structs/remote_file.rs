// What a HEAD request says about a package file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFile {
    pub length: u64,
    pub etag: String,
    pub last_modified: String,
}

impl RemoteFile {
    // the text kept next to a partial download
    pub fn to_meta(&self) -> String {
        format!("length={}\netag={}\nlast_modified={}\n", self.length, self.etag, self.last_modified)
    }

    pub fn from_meta(text: &str) -> Option<RemoteFile> {
        let mut length = None;
        let mut etag = None;
        let mut last_modified = None;
        for line in text.lines() {
            let (key, value) = line.split_once('=')?;
            match key {
                "length" => length = value.parse().ok(),
                "etag" => etag = Some(value.to_string()),
                "last_modified" => last_modified = Some(value.to_string()),
                _ => {}
            }
        }
        Some(RemoteFile { length: length?, etag: etag?, last_modified: last_modified? })
    }
}

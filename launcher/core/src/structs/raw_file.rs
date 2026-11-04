// A registry file as the client got it, before it is parsed.
#[derive(Debug, Clone)]
pub struct RawFile {
    pub bytes: Vec<u8>,
    pub etag: Option<String>,
    // the registry could not be reached and the disk copy was used
    pub stale: bool,
    // the bytes came from the network just now, so they are worth saving
    pub fresh: bool,
}

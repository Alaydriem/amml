use std::path::Path;

pub struct FileUri;

impl FileUri {
    // `file:///C:/dir/name.ext` with every byte outside the unreserved set (and `/`, `:`) percent-encoded
    pub fn from_path(path: &Path) -> String {
        let text = path.to_string_lossy().replace('\\', "/");
        let text = text.strip_prefix("//?/").unwrap_or(&text);
        let mut uri = String::from("file:///");
        for byte in text.trim_start_matches('/').bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                    uri.push(byte as char)
                }
                other => uri.push_str(&format!("%{other:02X}")),
            }
        }
        uri
    }
}

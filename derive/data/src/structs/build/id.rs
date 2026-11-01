#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildId {
    pub timestamp: u32,
    pub text_hash: u64,
}

impl BuildId {
    pub fn new(timestamp: u32, text_hash: u64) -> Self {
        Self { timestamp, text_hash }
    }

    pub fn file_stem(&self) -> String {
        format!("{:08x}-{:016x}", self.timestamp, self.text_hash)
    }
}

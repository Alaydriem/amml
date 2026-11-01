use amml_data::BuildId;

use super::{EntrySignature, SignatureStatus};

/// The signatures of every hooked entry of one build, in recipe order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureFile {
    pub build: BuildId,
    pub entries: Vec<EntrySignature>,
}

impl SignatureFile {
    pub fn new(build: BuildId, entries: Vec<EntrySignature>) -> Self {
        Self { build, entries }
    }

    pub fn unknown_count(&self) -> usize {
        self.entries.iter().filter(|entry| matches!(entry.signature.status, SignatureStatus::Unknown(_))).count()
    }

    /// 0 when every entry is known, 1 when any entry is unknown.
    pub fn exit_code(&self) -> u8 {
        u8::from(self.unknown_count() > 0)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "signatures {}: {} hooked entries, {} unknown",
            self.build.file_stem(),
            self.entries.len(),
            self.unknown_count()
        )
    }
}

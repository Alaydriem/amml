/// How one entry's signature on a build compares with the same entry on another build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureComparison {
    Same,
    // one line per difference, such as `+ r8 (32-bit)`
    Changed(Vec<String>),
    // a side could not be read, or the entry is missing from one side
    Unknown(String),
}

impl SignatureComparison {
    pub fn line(&self) -> String {
        match self {
            Self::Same => "same".to_string(),
            Self::Changed(changes) => format!("changed: {}", changes.join(", ")),
            Self::Unknown(reason) => format!("unknown: {reason}"),
        }
    }
}

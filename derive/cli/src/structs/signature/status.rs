#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureStatus {
    Known,
    // why the probe could not read the function
    Unknown(String),
}

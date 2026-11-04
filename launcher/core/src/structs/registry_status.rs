#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryStatus {
    Ok,
    // served from the disk cache because the registry could not be reached
    Stale { reason: String },
    // not reachable and nothing cached
    Offline,
    // reached, but the answer breaks a rule (size, schema, shape)
    Broken { reason: String },
}

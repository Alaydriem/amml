use crate::structs::LeaseEntry;

/// The leases in the order they were taken, and the lease policy: the mod that owns it, the ranking of mod
/// ids (best first) and the pairs of mod ids whose shared and exclusive leases run together.
pub(crate) struct LeaseBookState {
    pub(crate) leases: Vec<LeaseEntry>,
    pub(crate) policy_owner: Option<usize>,
    pub(crate) ranking: Vec<String>,
    pub(crate) coexist: Vec<(String, String)>,
}

/// One table offered by a mod, under a name and a major version.
#[derive(Debug)]
pub(crate) struct ProviderEntry<T> {
    pub(crate) name: String,
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) owner: usize,
    pub(crate) table: T,
}

use amml_abi::AmmlContainerEntry;

/// One entry of an opened contents list: the entry the caller reads, the type name it asks for apart, and whether the
/// caller took it.
pub struct ContentsEntry {
    pub entry: AmmlContainerEntry,
    pub name: String,
    pub taken: bool,
}

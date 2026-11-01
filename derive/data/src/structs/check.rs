use super::EntryId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    FunctionStart,
    // the value is the start of code that is called directly: 16-byte aligned, after int3 padding or a ret / jmp
    // that ends a `.pdata` range, inside no `.pdata` range unless it starts a primary one, and the target of at
    // least one `call rel32`. Holds for a leaf without `.pdata` as well as for a `.pdata` start.
    LeafStart,
    InVtableOf { class: String },
    // the value is the code target of a slot in `min_slot..=max_slot` of the vtable that the named `global` entry gives
    InVtable { vtable: EntryId, min_slot: u32, max_slot: u32 },
    // the value, as an RVA, lies inside the section of this exact name
    Section { name: String },
    Range { min: u64, max: u64 },
    // the named `field` entry resolved to exactly `value`; it lets an entry exist only for one code layout
    EntryIs { entry: EntryId, value: u64 },
}

impl Check {
    /// The entry this check reads, if any: the `global` of `in_vtable`, the `field` of `entry_is`.
    pub fn referenced_entry(&self) -> Option<&EntryId> {
        match self {
            Self::InVtable { vtable, .. } => Some(vtable),
            Self::EntryIs { entry, .. } => Some(entry),
            _ => None,
        }
    }
}

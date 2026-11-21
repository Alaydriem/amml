/// The plain fields of one item stack, as `AmmlInventoryTable::item_read` writes them. A `has_*` byte of 0
/// says the loader cannot read that field on this build, and the field is 0. The type name is read apart
/// (`item_name`).
///
/// The struct only grows: a field added later is appended, `size` and `version` grow, and the loader writes only
/// as many bytes as the caller's `size` covers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmlItemSnapshot {
    pub size: u32,
    pub version: u32,
    pub count: u8,
    pub has_aux: u8,
    pub has_max_count: u8,
    pub has_damage: u8,
    pub has_max_damage: u8,
    pub has_glint: u8,
    pub has_identity: u8,
    pub is_glint: u8,
    pub aux: i16,
    pub max_count: u8,
    pub _reserved: [u8; 5],
    pub damage: i32,
    pub max_damage: i32,
    pub identity: u64,
}

impl AmmlItemSnapshot {
    /// `size` of a v1 snapshot. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 40;

    /// An empty v1 snapshot for the loader to fill.
    pub const fn empty() -> Self {
        Self {
            size: Self::SIZE_V1,
            version: 1,
            count: 0,
            has_aux: 0,
            has_max_count: 0,
            has_damage: 0,
            has_max_damage: 0,
            has_glint: 0,
            has_identity: 0,
            is_glint: 0,
            aux: 0,
            max_count: 0,
            _reserved: [0; 5],
            damage: 0,
            max_damage: 0,
            identity: 0,
        }
    }
}

const _: () = assert!(core::mem::size_of::<AmmlItemSnapshot>() == AmmlItemSnapshot::SIZE_V1 as usize);

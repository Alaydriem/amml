use amml_abi::AmmlItemSnapshot;

/// The plain fields of one item stack, copied in one call by `ItemView::snapshot`. It holds no game pointer, so it
/// may be kept and sent to any thread. A field the loader cannot read on this build is `None`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemSnapshot {
    pub type_name: Option<String>,
    pub count: u8,
    pub aux: Option<i16>,
    pub max_count: Option<u8>,
    pub damage: Option<i32>,
    pub max_damage: Option<i32>,
    pub is_glint: Option<bool>,
    pub identity: Option<u64>,
    /// `ItemView::change_stamp`; always `None` for an entry of `ItemView::container_contents`.
    pub change_stamp: Option<u64>,
}

impl ItemSnapshot {
    /// The snapshot of the loader's fields and the name read apart.
    pub(crate) fn from_raw(raw: &AmmlItemSnapshot, type_name: Option<String>) -> Self {
        let when = |has: u8| has != 0;
        Self {
            type_name,
            count: raw.count,
            aux: when(raw.has_aux).then_some(raw.aux),
            max_count: when(raw.has_max_count).then_some(raw.max_count),
            damage: when(raw.has_damage).then_some(raw.damage),
            max_damage: when(raw.has_max_damage).then_some(raw.max_damage),
            is_glint: when(raw.has_glint).then_some(raw.is_glint != 0),
            identity: when(raw.has_identity).then_some(raw.identity),
            change_stamp: None,
        }
    }
}

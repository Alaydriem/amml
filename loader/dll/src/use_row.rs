use amml_abi::AmmlUses;

/// One row of the `uses` block a v3 mod reports from `amml_mod_load`: an event (`kind` 0) or an
/// area (`kind` 1) called `name`, used by the mod feature `feature`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseRow {
    pub feature: String,
    pub kind: u8,
    pub name: String,
    pub version: u32,
    pub required: bool,
}

impl UseRow {
    const MAX_ROWS: usize = 1024;

    /// Copies the rows of `uses`. A block shorter than its v1 size gives no rows; rows past
    /// `MAX_ROWS` and rows whose text is not UTF-8 are left out.
    ///
    /// # Safety
    /// `uses` points to a readable `AmmlUses` whose `rows` point to `count` readable rows, with
    /// string views valid for the call.
    pub unsafe fn read_all(uses: *const AmmlUses) -> Vec<Self> {
        // SAFETY: the caller guarantees `uses` is readable.
        let Some(uses) = (unsafe { uses.as_ref() }) else {
            return Vec::new();
        };
        if uses.size < AmmlUses::SIZE_V1 || uses.rows.is_null() {
            return Vec::new();
        }
        let count = uses.count.min(Self::MAX_ROWS);
        // SAFETY: the caller guarantees `rows` holds `count` rows; `count` is never above that.
        let rows = unsafe { std::slice::from_raw_parts(uses.rows, count) };
        rows.iter()
            .filter_map(|row| {
                // SAFETY: the caller guarantees the views are valid for the call.
                let (feature, name) = unsafe { (row.feature.as_str().ok()?, row.name.as_str().ok()?) };
                Some(Self {
                    feature: feature.to_string(),
                    kind: row.kind,
                    name: name.to_string(),
                    version: row.version,
                    required: row.required != 0,
                })
            })
            .collect()
    }
}

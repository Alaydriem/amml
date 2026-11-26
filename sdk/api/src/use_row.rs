use amml_abi::{AmmlStr, AmmlUseRow};

/// One `AmmlUseRow` whose string views point to `'static` strings. Only `Uses::event_row` and
/// `Uses::area_row` build one, so safe code cannot hand the loader or `Uses::requires` a row with a
/// wild pointer.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct UseRow(AmmlUseRow);

impl UseRow {
    pub(crate) const fn event(feature: &'static str, name: &'static str, version: u32, required: bool) -> Self {
        Self(AmmlUseRow {
            feature: AmmlStr { ptr: feature.as_ptr(), len: feature.len() },
            kind: AmmlUseRow::KIND_EVENT,
            name: AmmlStr { ptr: name.as_ptr(), len: name.len() },
            version,
            required: required as u8,
        })
    }

    pub(crate) const fn area(feature: &'static str, name: &'static str, version: u32, required: bool) -> Self {
        Self(AmmlUseRow {
            feature: AmmlStr { ptr: feature.as_ptr(), len: feature.len() },
            kind: AmmlUseRow::KIND_AREA,
            name: AmmlStr { ptr: name.as_ptr(), len: name.len() },
            version,
            required: required as u8,
        })
    }

    /// The feature this row belongs to.
    pub(crate) fn feature(&self) -> &'static str {
        Self::text(self.0.feature)
    }

    /// The event or area id.
    pub(crate) fn name(&self) -> &'static str {
        Self::text(self.0.name)
    }

    /// Whether this row names an area (`AmmlUseRow::KIND_AREA`) rather than an event.
    pub(crate) fn is_area(&self) -> bool {
        self.0.kind == AmmlUseRow::KIND_AREA
    }

    pub(crate) fn version(&self) -> u32 {
        self.0.version
    }

    pub(crate) fn required(&self) -> bool {
        self.0.required != 0
    }

    // A view built by `event` or `area` from a `&'static str`.
    fn text(view: AmmlStr) -> &'static str {
        // SAFETY: only `event` and `area` build a `UseRow`, and both take the views from `&'static str`,
        // so the bytes are valid UTF-8 for `'static`.
        #[allow(unsafe_code)]
        let text = unsafe { view.as_str() };
        text.unwrap_or("")
    }

    /// The ABI rows of `rows`, for the info block.
    pub(crate) const fn as_abi(rows: &'static [UseRow]) -> &'static [AmmlUseRow] {
        // SAFETY: `UseRow` is `#[repr(transparent)]` over `AmmlUseRow`, so a slice of one has the
        // layout of a slice of the other, with the same length and lifetime.
        #[allow(unsafe_code)]
        unsafe {
            core::slice::from_raw_parts(rows.as_ptr().cast::<AmmlUseRow>(), rows.len())
        }
    }
}

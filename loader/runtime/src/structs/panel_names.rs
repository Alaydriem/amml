use amml_abi::{AmmlScreenKind, AmmlUiProfile};

/// The names of the controls that lead to the content panel of a screen, the rectangle that `content_rect`
/// reports.
pub struct PanelNames;

impl PanelNames {
    /// The first control, found depth first from the root, that every path starts under.
    pub const CONTENT: &'static str = "root_screen_panel";

    /// The child names to follow under `CONTENT`, one level each, to the content panel of `kind` in `profile`.
    /// `None` for a kind without a known panel and for any profile but the classic one.
    pub fn path(kind: AmmlScreenKind, profile: AmmlUiProfile) -> Option<&'static [&'static str]> {
        if profile != AmmlUiProfile::CLASSIC {
            return None;
        }
        match kind {
            AmmlScreenKind::SMALL_CHEST
            | AmmlScreenKind::LARGE_CHEST
            | AmmlScreenKind::BARREL
            | AmmlScreenKind::SHULKER_BOX
            | AmmlScreenKind::ENDER_CHEST
            | AmmlScreenKind::HOPPER
            | AmmlScreenKind::DISPENSER
            | AmmlScreenKind::DROPPER => Some(&["root_panel"]),
            AmmlScreenKind::PLAYER_INVENTORY => Some(&["content_stack_panel", "player_inventory"]),
            _ => None,
        }
    }
}

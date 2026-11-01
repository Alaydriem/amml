use amml_i18n::text::Text;

/// What a menu item shows at its right end.
#[derive(Debug, Clone, Default)]
pub enum MenuTrailing {
    #[default]
    None,
    /// The item's shortcut, as caption text in `--color-text-faint` (Paper: "Ctrl+D").
    Shortcut(Text),
    /// The CHEVRON of an item that opens a submenu.
    Submenu,
    /// A small check: the current choice in a submenu (scene colour).
    Check,
}

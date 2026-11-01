use amml_i18n::text::Text;

/// The Original row at the top of the scene switcher list: the replay as recorded, with no scene.
pub struct SceneSwitcherOriginal {
    pub name: Text,
    /// What the Original keeps (no overrides, the field of view).
    pub detail: Text,
    pub active: bool,
}

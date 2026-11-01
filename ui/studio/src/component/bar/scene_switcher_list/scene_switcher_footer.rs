use amml_i18n::text::Text;
use amml_ui::icon::Icon;

/// One action under the scene switcher list (new scene, duplicate).
pub struct SceneSwitcherFooter {
    pub icon: Icon,
    pub label: Text,
    /// The key chord, when the action has one.
    pub shortcut: Option<Text>,
}

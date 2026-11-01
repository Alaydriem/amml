use amml_i18n::text::Text;

/// What a confirm dialog says: its title (as in "Delete “Portal run”?"), the line under it, the body, and the
/// words on the danger button (as in "Delete scene").
#[derive(Debug, Clone)]
pub struct ConfirmDialogView {
    pub title: Text,
    pub subtitle: Text,
    pub body: Text,
    pub confirm: Text,
}

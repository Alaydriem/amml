use amml_i18n::text::Text;

/// The undo steps, oldest first, each named as the caller writes it (as in "Move keys").
#[derive(Debug, Clone)]
pub struct UndoHistoryView {
    pub steps: Vec<Text>,
}

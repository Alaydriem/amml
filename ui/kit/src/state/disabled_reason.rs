use amml_i18n::message::Message;
use amml_i18n::text::Text;

use crate::widget::Tooltip;

/// Why a control is disabled. The reason shows as the control's tooltip (Paper: Tooltip sheet,
/// DISABLED CONTROL), at full opacity.
#[derive(Debug, Clone)]
pub struct DisabledReason(Text);

impl DisabledReason {
    pub fn new(text: Text) -> Self {
        Self(text)
    }

    pub fn message(message: impl Message) -> Self {
        Self(Text::message(message))
    }

    pub fn text(&self) -> &Text {
        &self.0
    }

    pub fn tooltip(&self) -> Tooltip {
        Tooltip::new(self.0.clone())
    }
}

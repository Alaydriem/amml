mod confirm_action;
mod confirm_dialog_view;

pub use confirm_action::ConfirmAction;
pub use confirm_dialog_view::ConfirmDialogView;

use amml_i18n::text::Text;
use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ButtonKind, ModalFrame, ModalResponse, TextButton};
use egui::{Context, Id, Ui};

use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

/// Paper: Menus + Dialogs, `Confirm delete (scene, track)` (`9PB-0`). A question in the amml-ui `ModalFrame`: the
/// body, then Cancel and the danger button.
pub struct ConfirmDialog<'a> {
    view: &'a ConfirmDialogView,
}

impl<'a> ConfirmDialog<'a> {
    pub fn new(view: &'a ConfirmDialogView) -> Self {
        Self { view }
    }

    /// Shows the dialog over the screen.
    pub fn show(&self, ctx: &Context) -> Option<ConfirmAction> {
        let mut confirm = None;
        let response = self.frame().show(ctx, |ui| self.body(ui), |ui| confirm = self.footer(ui));
        Self::action(response, confirm)
    }

    /// Shows the card where `ui` places it, for the gallery.
    pub fn show_card(&self, ui: &mut Ui) -> Option<ConfirmAction> {
        let mut confirm = None;
        let response = self.frame().show_card(ui, |ui| self.body(ui), |ui| confirm = self.footer(ui));
        Self::action(response, confirm)
    }

    fn frame(&self) -> ModalFrame {
        ModalFrame::new(Id::unique(StudioUiId::ConfirmDialog), self.view.title.clone())
            .subtitle(self.view.subtitle.clone())
    }

    fn body(&self, ui: &mut Ui) {
        let style = TextStyle::new(TypeSize::Help, FontFace::Regular);
        let ink = <color::Text as ColorToken>::VALUE;
        let text = UiText::resolve(ui.ctx(), &self.view.body);
        let galley = Draw::galley(ui.painter(), &text, style, ink, TextFit::Wrap(ui.available_width()));
        let (rect, _) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        ui.painter().galley(rect.min, galley, ink);
    }

    // the danger button, then Cancel (the footer runs right to left)
    fn footer(&self, ui: &mut Ui) -> Option<ConfirmAction> {
        let confirm = TextButton::new(ButtonKind::Danger, self.view.confirm.clone()).show(ui).clicked();
        let cancel = TextButton::new(ButtonKind::Ghost, Text::message(StudioUiMessage::Cancel)).show(ui).clicked();
        if confirm { Some(ConfirmAction::Confirm) } else { cancel.then_some(ConfirmAction::Cancel) }
    }

    fn action(response: ModalResponse, footer: Option<ConfirmAction>) -> Option<ConfirmAction> {
        if response.closed { Some(ConfirmAction::Cancel) } else { footer }
    }
}

mod rename_action;

pub use rename_action::RenameAction;

use std::cell::RefCell;

use amml_i18n::text::Text;
use amml_ui::state::DisabledReason;
use amml_ui::widget::{ButtonKind, ModalFrame, ModalResponse, TextButton, TextField, TextFieldKind};
use egui::{Context, Id, Key, Ui};

use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

/// Paper: Menus + Dialogs, `Rename (scene, bookmark, preset)` (`9QD-0`). A name field in the amml-ui `ModalFrame`,
/// focused when it opens. Enter or Rename applies a name that is not empty; Escape cancels. The caller keeps the
/// text in `value`. A caller can give its own hint and a problem with the name; a problem replaces the hint and keeps
/// the Rename button off.
pub struct RenameDialog<'a> {
    title: &'a Text,
    value: &'a mut String,
    field: Id,
    hint: Option<Text>,
    problem: Option<Text>,
}

impl<'a> RenameDialog<'a> {
    /// Paper: the rename cards are 460 wide.
    const WIDTH: f32 = 460.0;

    pub fn new(title: &'a Text, value: &'a mut String) -> Self {
        Self { title, value, field: Id::unique(StudioUiId::RenameField), hint: None, problem: None }
    }

    /// Another id for the field, for a second dialog on one screen (the gallery shows three).
    pub fn id(mut self, id: Id) -> Self {
        self.field = id;
        self
    }

    /// The line under the title, in place of the standard hint.
    pub fn hint(mut self, hint: Text) -> Self {
        self.hint = Some(hint);
        self
    }

    /// What is wrong with the name typed now (`None` when nothing is). It shows under the title in place of the
    /// hint, and Enter and the Rename button do nothing while it is set.
    pub fn problem(mut self, problem: Option<Text>) -> Self {
        self.problem = problem;
        self
    }

    pub fn show(self, ctx: &Context) -> Option<RenameAction> {
        let frame = self.frame();
        let field = self.field;
        let problem = self.problem.clone();
        // the body edits the text and the footer reads it, one after the other
        let value = RefCell::new(self.value);
        let (mut body, mut footer) = (None, None);
        let response = frame.show(
            ctx,
            |ui| body = Self::body(ui, field, &mut value.borrow_mut(), problem.as_ref()),
            |ui| footer = Self::footer(ui, &value.borrow(), problem.as_ref()),
        );
        Self::action(response, body.or(footer))
    }

    /// Shows the card where `ui` places it, for the gallery.
    pub fn show_card(self, ui: &mut Ui) -> Option<RenameAction> {
        let frame = self.frame();
        let field = self.field;
        let problem = self.problem.clone();
        let value = RefCell::new(self.value);
        let (mut body, mut footer) = (None, None);
        let response = frame.show_card(
            ui,
            |ui| body = Self::body(ui, field, &mut value.borrow_mut(), problem.as_ref()),
            |ui| footer = Self::footer(ui, &value.borrow(), problem.as_ref()),
        );
        Self::action(response, body.or(footer))
    }

    fn frame(&self) -> ModalFrame {
        let subtitle = self
            .problem
            .clone()
            .or_else(|| self.hint.clone())
            .unwrap_or_else(|| Text::message(StudioUiMessage::RenameHint));
        ModalFrame::new(Id::unique(StudioUiId::RenameDialog), self.title.clone()).subtitle(subtitle).width(Self::WIDTH)
    }

    // the field, focused when nothing else has the focus; Enter applies and Escape cancels
    fn body(ui: &mut Ui, field: Id, value: &mut String, problem: Option<&Text>) -> Option<RenameAction> {
        if ui.memory(|memory| memory.focused().is_none()) {
            ui.memory_mut(|memory| memory.request_focus(field));
        }
        let placeholder = Text::message(StudioUiMessage::RenamePlaceholder);
        let response = TextField::new(field, value, TextFieldKind::Plain, placeholder).show(ui, ui.available_width());
        if !response.lost_focus() {
            return None;
        }
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            Some(RenameAction::Cancel)
        } else if ui.input(|input| input.key_pressed(Key::Enter)) && !value.trim().is_empty() && problem.is_none() {
            Some(RenameAction::Confirm(value.trim().to_owned()))
        } else {
            None
        }
    }

    fn footer(ui: &mut Ui, value: &str, problem: Option<&Text>) -> Option<RenameAction> {
        let empty = value.trim().is_empty();
        let off = empty || problem.is_some();
        let reason = match problem {
            Some(problem) => DisabledReason::new(problem.clone()),
            None => DisabledReason::message(StudioUiMessage::NameIsEmpty),
        };
        let button = TextButton::new(ButtonKind::Primary, Text::message(StudioUiMessage::Rename)).enabled(!off, reason);
        (button.show(ui).clicked() && !off).then(|| RenameAction::Confirm(value.trim().to_owned()))
    }

    fn action(response: ModalResponse, action: Option<RenameAction>) -> Option<RenameAction> {
        if response.closed { Some(RenameAction::Cancel) } else { action }
    }
}

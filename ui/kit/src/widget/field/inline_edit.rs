use egui::{Align2, Color32, Direction, Frame, Id, Key, Layout, Rect, RichText, TextEdit, Ui, UiBuilder};

use super::InlineOutcome;
use crate::draw::TextStyle;
use crate::id::AmmlUiId;
use crate::tokens::{ColorToken, color};

/// Typing into a field that otherwise shows a formatted value (drag field, colour field). The typed
/// text lives in egui's temporary data under the field's id, so two fields never share it.
///
/// Typing starts empty, with the current value as faint hint text, so the first key replaces the
/// value, as a select-all would. egui 0.37's `TextEdit` cannot keep a selection made before it has
/// focus: each pass collapses it to one caret.
pub(crate) struct InlineEdit;

impl InlineEdit {
    pub fn is_editing(ui: &Ui, id: Id) -> bool {
        ui.data(|data| data.get_temp::<String>(Self::buffer_id(id))).is_some()
    }

    /// Starts typing and takes the keyboard focus on a later pass.
    pub fn start(ui: &Ui, id: Id) {
        ui.data_mut(|data| data.insert_temp(Self::buffer_id(id), String::new()));
        // focus asked for in this pass would be dropped at its end: the click that started typing
        // was not on the edit
        let pass = ui.ctx().cumulative_pass_nr();
        ui.data_mut(|data| data.insert_temp(Self::focus_id(id), pass));
    }

    /// Shows the text edit in `rect` while typing, and says what the user did. `current` is the
    /// value as it shows now: the faint hint, and the edit's accessible name.
    pub fn show(
        ui: &mut Ui,
        id: Id,
        rect: Rect,
        style: TextStyle,
        color: Color32,
        align: Align2,
        current: String,
    ) -> InlineOutcome {
        let Some(mut buffer) = ui.data(|data| data.get_temp::<String>(Self::buffer_id(id))) else {
            return InlineOutcome::Idle;
        };
        let started = ui.data(|data| data.get_temp::<u64>(Self::focus_id(id)));
        if started.is_some_and(|pass| pass < ui.ctx().cumulative_pass_nr()) {
            ui.data_mut(|data| data.remove::<u64>(Self::focus_id(id)));
            ui.memory_mut(|memory| memory.request_focus(Self::edit_id(id)));
        }
        let escape = ui.input(|input| input.key_pressed(Key::Escape));
        let hint = RichText::new(current).font(style.font_id()).color(<color::TextFaint as ColorToken>::VALUE);
        let edit = TextEdit::singleline(&mut buffer)
            .id(Self::edit_id(id))
            .frame(Frame::NONE)
            .margin(egui::Margin::ZERO)
            .font(style.font_id())
            .text_color(color)
            .hint_text(hint)
            .align(align)
            .desired_width(rect.width());
        // a child ui, not `put`: `put` moves the parent cursor, which shifts the widgets after the field
        let layout = Layout::centered_and_justified(Direction::TopDown);
        let response = ui.new_child(UiBuilder::new().max_rect(rect).layout(layout)).add(edit);
        // Escape, or leaving the field without typing, keeps the value
        if escape || (response.lost_focus() && buffer.trim().is_empty()) {
            Self::stop(ui, id);
            return InlineOutcome::Cancel;
        }
        if response.lost_focus() {
            Self::stop(ui, id);
            return InlineOutcome::Commit(buffer);
        }
        ui.data_mut(|data| data.insert_temp(Self::buffer_id(id), buffer));
        InlineOutcome::Editing
    }

    fn stop(ui: &Ui, id: Id) {
        ui.data_mut(|data| data.remove::<String>(Self::buffer_id(id)));
    }

    fn buffer_id(id: Id) -> Id {
        id.with(AmmlUiId::InlineBuffer)
    }

    fn edit_id(id: Id) -> Id {
        id.with(AmmlUiId::InlineEdit)
    }

    fn focus_id(id: Id) -> Id {
        id.with(AmmlUiId::InlineFocus)
    }
}

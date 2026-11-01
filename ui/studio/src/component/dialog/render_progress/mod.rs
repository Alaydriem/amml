mod render_progress_action;
mod render_progress_view;

pub use render_progress_action::RenderProgressAction;
pub use render_progress_view::RenderProgressView;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{ButtonKind, ModalFrame, ModalResponse, StatusPill, TextButton};
use egui::{Align2, Context, Id, Rect, Sense, Ui, pos2, vec2};

use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

/// Paper: Menus + Dialogs, `Dialog / Render progress — rendering, paused, failed`. The card in the amml-ui `ModalFrame` (the
/// default `--container-dialog` width) over the editor while a render runs: the status pill and the frame count, the
/// progress bar, and the time left or the result, `--spacing-s` apart; then Open queue at the left, and Cancel while it
/// runs, else Close. Escape and the close mark cancel a running render. A click beside the card does nothing: the
/// render goes on.
pub struct RenderProgressCard<'a> {
    view: &'a RenderProgressView,
}

impl<'a> RenderProgressCard<'a> {
    /// Paper: Render progress sheet, the bar height (as the render queue's bar, `9A9-0`).
    const BAR: f32 = 6.0;

    pub fn new(view: &'a RenderProgressView) -> Self {
        Self { view }
    }

    pub fn show(&self, ctx: &Context) -> Option<RenderProgressAction> {
        let mut footer = None;
        let response = self.frame().show(ctx, |ui| self.body(ui), |ui| footer = self.footer(ui));
        // a press beside the card is the scrim, not a request to stop
        let beside = ctx.input(|input| {
            input.pointer.any_click() && input.pointer.interact_pos().is_some_and(|at| !response.card.contains(at))
        });
        self.action(response, footer, beside)
    }

    /// Shows the card where `ui` places it, for the gallery and the tests.
    pub fn show_card(&self, ui: &mut Ui) -> Option<RenderProgressAction> {
        let mut footer = None;
        let response = self.frame().show_card(ui, |ui| self.body(ui), |ui| footer = self.footer(ui));
        self.action(response, footer, false)
    }

    fn frame(&self) -> ModalFrame {
        ModalFrame::new(Id::unique(StudioUiId::RenderProgress), self.view.title.clone())
            .subtitle(self.view.file.clone())
    }

    // the pill and the frame count, the bar, then the time left or the result
    fn body(&self, ui: &mut Ui) {
        let view = self.view;
        let width = ui.available_width();
        // Paper: the status line is as tall as the pill, `--spacing-control-small`
        let line = <spacing::ControlSmall as SpacingToken>::VALUE;
        let gap = <spacing::S as SpacingToken>::VALUE;
        let caption = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        // Paper: a failed render's reason is `--color-danger`
        let dim = if view.status == crate::component::dialog::JobStatus::Failed {
            <color::Danger as ColorToken>::VALUE
        } else {
            <color::TextDim as ColorToken>::VALUE
        };
        // Paper: the line under the bar wraps, so a long reason or pause line shows whole
        let note =
            Draw::galley(ui.painter(), &UiText::resolve(ui.ctx(), &view.time_left), caption, dim, TextFit::Wrap(width));
        let height = line + gap * 2.0 + Self::BAR + note.size().y.max(TypeSize::Caption.line_height());
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let painter = ui.painter().clone();
        let pill = StatusPill::new(view.status.tone(), view.status.label());
        let pill_rect = pill.paint(&painter, pos2(rect.left(), rect.top() + line / 2.0), Align2::LEFT_CENTER);
        let semibold = TextStyle::new(TypeSize::Note, FontFace::Semibold);
        let ink = <color::Text as ColorToken>::VALUE;
        Draw::text(
            &painter,
            pos2(rect.right(), pill_rect.center().y),
            Align2::RIGHT_CENTER,
            &view.frames,
            semibold,
            ink,
            TextFit::Line,
        );
        let track = Rect::from_min_size(pos2(rect.left(), rect.top() + line + gap), vec2(rect.width(), Self::BAR));
        Draw::fill(&painter, track, Corner::Full, <color::BgField as ColorToken>::VALUE);
        let percent = if view.progress.is_nan() { 0.0 } else { view.progress.clamp(0.0, 1.0) };
        if percent > 0.0 {
            let fill = Rect::from_min_size(track.min, vec2(track.width() * percent, Self::BAR));
            Draw::fill(&painter, fill, Corner::Full, <color::Accent as ColorToken>::VALUE);
        }
        painter.galley(pos2(rect.left(), track.bottom() + gap), note, dim);
    }

    // Cancel while the render runs, else Close; then Open queue at the left (the footer runs right to left)
    fn footer(&self, ui: &mut Ui) -> Option<RenderProgressAction> {
        let end = if self.view.running {
            let cancel = TextButton::new(ButtonKind::Outline, Text::message(StudioUiMessage::Cancel));
            cancel.show(ui).clicked().then_some(RenderProgressAction::Cancel)
        } else {
            let close = TextButton::new(ButtonKind::Primary, Text::message(StudioUiMessage::Close));
            close.show(ui).clicked().then_some(RenderProgressAction::Close)
        };
        let open = TextButton::new(ButtonKind::Ghost, Text::message(StudioUiMessage::OpenQueue));
        ui.add_space((ui.available_width() - open.desired_width(ui.ctx())).max(0.0));
        let open = open.show(ui).clicked().then_some(RenderProgressAction::OpenQueue);
        end.or(open)
    }

    fn action(
        &self,
        response: ModalResponse,
        action: Option<RenderProgressAction>,
        beside: bool,
    ) -> Option<RenderProgressAction> {
        if response.closed && !beside {
            Some(if self.view.running { RenderProgressAction::Cancel } else { RenderProgressAction::Close })
        } else {
            action
        }
    }
}

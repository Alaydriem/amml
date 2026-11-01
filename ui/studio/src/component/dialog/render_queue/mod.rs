mod job_status;
mod render_job_row;
mod render_queue_action;
mod render_queue_view;

pub use job_status::JobStatus;
pub use render_job_row::RenderJobRow;
pub use render_queue_action::RenderQueueAction;
pub use render_queue_view::RenderQueueView;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::state::InteractionState;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{ButtonKind, EmptyState, IconButton, Leading, ModalFrame, ModalResponse, StatusPill, TextButton};
use egui::{Align2, Context, Id, Rect, Sense, Ui, UiBuilder, pos2, vec2};

use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

// a row's trailing button: its look, its label and the action it asks for
type RowButton = (ButtonKind, StudioUiMessage, fn(usize) -> RenderQueueAction);

/// Paper: Menus + Dialogs, `Render queue` (`9A9-0` rendering, `9D1-0` paused, `Dialog / Render queue — preparing,
/// finishing, reorder menu`). The jobs in the amml-ui `ModalFrame`: a thumbnail with the scene's colour, the name,
/// settings and file, and the status (a progress bar while rendering, else a status pill and its detail; a failed
/// job's reason under its pill), with Retry or Show file where they apply. The trailing button cancels a job that is
/// not over (stop) and removes one that is (close). A right click on a row asks for the job menu. The footer has the
/// audio note (info mark) at the left.
pub struct RenderQueue<'a> {
    view: &'a RenderQueueView,
}

impl<'a> RenderQueue<'a> {
    /// Paper: the queue card is 820 wide.
    const WIDTH: f32 = 820.0;
    // Paper: rows are 84 high, 8 apart, 12 in, parts 14 apart; the thumbnail 104 × 58 with a 2 px scene bar 6 in
    // and 2 up
    const ROW: f32 = 84.0;
    const ROW_GAP: f32 = 8.0;
    const ROW_PADDING: f32 = 12.0;
    const PART_GAP: f32 = 14.0;
    const THUMB: [f32; 2] = [104.0, 58.0];
    const SCENE_BAR: f32 = 2.0;
    const SCENE_INSET: f32 = 6.0;
    // Paper: the status column is 254 wide, the bar 6 high and 6 from the lines over and under it; the trailing
    // button 28 square
    const STATUS: f32 = 254.0;
    const BAR: f32 = 6.0;
    const BAR_GAP: f32 = 6.0;
    const MENU: f32 = 28.0;
    // Paper: the status pill is 20 high; a failed job's reason is 8 under it; Retry is 26 high, and on a failed job
    // its top is 6 over the pill's
    const PILL: f32 = 20.0;
    const FAILED_GAP: f32 = 8.0;
    const RETRY: f32 = 26.0;
    const RETRY_RISE: f32 = 6.0;
    const LINE_GAP: f32 = 5.0;
    // Paper: the running job's border is `color-mix(var(--color-accent), transparent)`, the accent at half
    const RUNNING_ALPHA: f32 = 0.5;
    // Paper: the footer's info mark is 8 from the audio note
    const NOTE_GAP: f32 = 8.0;
    // the empty queue's height in the card
    const EMPTY: f32 = 240.0;

    pub fn new(view: &'a RenderQueueView) -> Self {
        Self { view }
    }

    pub fn show(&self, ctx: &Context) -> Option<RenderQueueAction> {
        let (mut body, mut footer) = (None, None);
        let response = self.frame().show(ctx, |ui| body = self.body(ui), |ui| footer = self.footer(ui));
        Self::action(response, body.or(footer))
    }

    /// Shows the card where `ui` places it, for the gallery.
    pub fn show_card(&self, ui: &mut Ui) -> Option<RenderQueueAction> {
        let (mut body, mut footer) = (None, None);
        let response = self.frame().show_card(ui, |ui| body = self.body(ui), |ui| footer = self.footer(ui));
        Self::action(response, body.or(footer))
    }

    fn frame(&self) -> ModalFrame {
        ModalFrame::new(Id::unique(StudioUiId::RenderQueue), Text::message(StudioUiMessage::RenderQueue))
            .subtitle(self.view.subtitle.clone())
            .width(Self::WIDTH)
    }

    fn body(&self, ui: &mut Ui) -> Option<RenderQueueAction> {
        // Paper packs the rows `ROW_GAP` apart, not the modal body's gap
        ui.spacing_mut().item_spacing.y = 0.0;
        if self.view.jobs.is_empty() {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::EMPTY), Sense::hover());
            let title = Text::message(StudioUiMessage::QueueEmpty);
            let message = Text::message(StudioUiMessage::QueueEmptyHint);
            EmptyState::new(Icon::Video, title, message).show(&mut ui.new_child(UiBuilder::new().max_rect(rect)));
            return None;
        }
        let mut action = None;
        for (index, job) in self.view.jobs.iter().enumerate() {
            let (row, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW), Sense::click());
            action = self.row(ui, row, index, job).or(action);
            if response.secondary_clicked() {
                let at = response.interact_pointer_pos().unwrap_or(row.center());
                action = Some(RenderQueueAction::OpenJobMenu(index, at));
            }
            if index + 1 < self.view.jobs.len() {
                ui.add_space(Self::ROW_GAP);
            }
        }
        action
    }

    fn row(&self, ui: &mut Ui, row: Rect, index: usize, job: &RenderJobRow) -> Option<RenderQueueAction> {
        let painter = ui.painter().clone();
        let border = if job.status.is_running() {
            <color::Accent as ColorToken>::VALUE.gamma_multiply(Self::RUNNING_ALPHA)
        } else {
            <color::Line as ColorToken>::VALUE
        };
        Draw::fill(&painter, row, Corner::Panel, <color::BgPanel as ColorToken>::VALUE);
        Draw::stroke(&painter, row, Corner::Panel, border);
        let inner = row.shrink2(vec2(Self::ROW_PADDING, 0.0));
        let thumb = Rect::from_min_size(
            pos2(inner.left(), row.center().y - Self::THUMB[1] / 2.0),
            vec2(Self::THUMB[0], Self::THUMB[1]),
        );
        Draw::fill(&painter, thumb, Corner::Control, <color::BgApp as ColorToken>::VALUE);
        let bar = Rect::from_min_max(
            pos2(thumb.left() + Self::SCENE_INSET, thumb.bottom() - Self::SCENE_BAR * 2.0),
            pos2(thumb.right() - Self::SCENE_INSET, thumb.bottom() - Self::SCENE_BAR),
        );
        Draw::fill(&painter, bar, Corner::Full, job.scene.color());
        let menu = Rect::from_min_size(
            pos2(inner.right() - Self::MENU, row.center().y - Self::MENU / 2.0),
            vec2(Self::MENU, Self::MENU),
        );
        let status = Rect::from_min_max(
            pos2(menu.left() - Self::PART_GAP - Self::STATUS, row.top()),
            pos2(menu.left() - Self::PART_GAP, row.bottom()),
        );
        let text = Rect::from_min_max(
            pos2(thumb.right() + Self::PART_GAP, row.top()),
            pos2(status.left() - Self::PART_GAP, row.bottom()),
        );
        Self::text(&painter, text, job);
        let mut action = self.status(ui, status, index, job);
        let (icon, word, act): (Icon, StudioUiMessage, fn(usize) -> RenderQueueAction) = if job.status.is_finished() {
            (Icon::Close, StudioUiMessage::RemoveJob, RenderQueueAction::Remove)
        } else {
            (Icon::Stop, StudioUiMessage::Cancel, RenderQueueAction::Cancel)
        };
        let button = IconButton::new(icon).tooltip(Text::message(word));
        let response = button.show(&mut ui.new_child(UiBuilder::new().max_rect(menu)));
        if response.clicked() {
            action = Some(act(index));
        }
        action
    }

    // the name, the settings line and the file name, stacked in the middle of the row
    fn text(painter: &egui::Painter, rect: Rect, job: &RenderJobRow) {
        let lines = [
            (&job.name, TextStyle::new(TypeSize::Label, FontFace::Semibold), <color::Text as ColorToken>::VALUE),
            (&job.meta, TextStyle::new(TypeSize::Note, FontFace::Regular), <color::TextDim as ColorToken>::VALUE),
            (&job.file, TextStyle::new(TypeSize::Caption, FontFace::Regular), <color::TextDim as ColorToken>::VALUE),
        ];
        let height: f32 =
            lines.iter().map(|(_, style, _)| style.size.line_height()).sum::<f32>() + Self::LINE_GAP * 2.0;
        let mut y = rect.center().y - height / 2.0;
        for (text, style, ink) in lines {
            Draw::text(
                painter,
                pos2(rect.left(), y),
                Align2::LEFT_TOP,
                text,
                style,
                ink,
                TextFit::Truncate(rect.width().max(0.0)),
            );
            y += style.size.line_height() + Self::LINE_GAP;
        }
    }

    // the status column: the bar while rendering, else the pill, the detail and Retry or Show file
    fn status(&self, ui: &mut Ui, rect: Rect, index: usize, job: &RenderJobRow) -> Option<RenderQueueAction> {
        let painter = ui.painter().clone();
        let caption = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        if job.status == JobStatus::Rendering {
            let percent = if job.progress.is_nan() { 0.0 } else { job.progress.clamp(0.0, 1.0) };
            let words = Text::format(
                StudioUiMessage::RenderingPercent,
                Args::new().uint("percent", (percent * 100.0).round() as u64),
            );
            let ink = <color::Text as ColorToken>::VALUE;
            let semibold = TextStyle::new(TypeSize::Note, FontFace::Semibold);
            let top = rect.center().y - Self::BAR / 2.0 - TypeSize::Note.line_height() - Self::BAR_GAP;
            let head =
                Draw::text(&painter, pos2(rect.left(), top), Align2::LEFT_TOP, &words, semibold, ink, TextFit::Line);
            let dim = <color::TextDim as ColorToken>::VALUE;
            Draw::text(
                &painter,
                pos2(rect.right(), head.center().y),
                Align2::RIGHT_CENTER,
                &job.detail,
                caption,
                dim,
                TextFit::Line,
            );
            let track = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - Self::BAR / 2.0),
                vec2(rect.width(), Self::BAR),
            );
            Draw::fill(&painter, track, Corner::Full, <color::BgField as ColorToken>::VALUE);
            if percent > 0.0 {
                let fill = Rect::from_min_size(track.min, vec2(track.width() * percent, Self::BAR));
                Draw::fill(&painter, fill, Corner::Full, <color::Accent as ColorToken>::VALUE);
            }
            let micro = TextStyle::new(TypeSize::Micro, FontFace::Regular);
            let faint = <color::TextFaint as ColorToken>::VALUE;
            Draw::text(
                &painter,
                pos2(rect.left(), track.bottom() + Self::BAR_GAP),
                Align2::LEFT_TOP,
                &job.note,
                micro,
                faint,
                TextFit::Truncate(rect.width()),
            );
            return None;
        }
        // Paper: a failed job stacks its reason under the pill, `FAILED_GAP` apart, and Retry sits on the pill's line
        let failed = job.status == JobStatus::Failed;
        let line = if failed {
            let stack = Self::PILL + Self::FAILED_GAP + TypeSize::Caption.line_height();
            rect.center().y - stack / 2.0 + Self::PILL / 2.0
        } else {
            rect.center().y
        };
        let pill = StatusPill::new(job.status.tone(), job.status.label());
        let pill_rect = pill.paint(&painter, pos2(rect.left(), line), Align2::LEFT_CENTER);
        let detail_ink = if failed {
            crate::rule::LabelTint::of(<color::Danger as ColorToken>::VALUE)
        } else {
            <color::TextFaint as ColorToken>::VALUE
        };
        let button: Option<RowButton> = match job.status {
            JobStatus::Failed | JobStatus::Cancelled => {
                Some((ButtonKind::Outline, StudioUiMessage::Retry, RenderQueueAction::Retry))
            }
            JobStatus::Done => Some((ButtonKind::Ghost, StudioUiMessage::ShowFile, RenderQueueAction::ShowFile)),
            _ => None,
        };
        let place = button.map(|(kind, word, _)| {
            let width = Self::button(kind, word).desired_width(ui.ctx());
            // Paper: Retry is 26 high (raised on a failed job), Show file 28
            let (top, height) = match job.status {
                JobStatus::Failed => (pill_rect.top() - Self::RETRY_RISE, Self::RETRY),
                JobStatus::Cancelled => (line - Self::RETRY / 2.0, Self::RETRY),
                _ => (line - Self::MENU / 2.0, Self::MENU),
            };
            Rect::from_min_size(pos2(rect.right() - width, top), vec2(width, height))
        });
        if failed {
            let at = pos2(rect.left(), pill_rect.bottom() + Self::FAILED_GAP);
            Draw::text(
                &painter,
                at,
                Align2::LEFT_TOP,
                &job.detail,
                caption,
                detail_ink,
                TextFit::Truncate(rect.width()),
            );
        } else {
            let at = pos2(pill_rect.right() + Self::LINE_GAP * 2.0, line);
            let end = place.map_or(rect.right(), |place| place.left() - Self::LINE_GAP);
            let room = (end - at.x).max(0.0);
            // Paper: the detail of a waiting or busy job sits at the right of the status column
            let (at, align) =
                if place.is_none() { (pos2(end, line), Align2::RIGHT_CENTER) } else { (at, Align2::LEFT_CENTER) };
            Draw::text(&painter, at, align, &job.detail, caption, detail_ink, TextFit::Truncate(room));
        }
        let ((kind, word, act), place) = button.zip(place)?;
        let button = Self::button(kind, word);
        // a child ui, so the button does not move the row cursor
        let mut child = ui.new_child(UiBuilder::new().max_rect(place));
        let response = child.allocate_rect(place, Sense::click());
        button.paint(&painter, place, InteractionState::of(&response, true));
        response.clicked().then(|| act(index))
    }

    // Paper: Show file leads with the folder mark
    fn button(kind: ButtonKind, word: StudioUiMessage) -> TextButton<'static> {
        let button = TextButton::new(kind, Text::message(word));
        if word == StudioUiMessage::ShowFile { button.leading(Leading::Icon(Icon::Folder)) } else { button }
    }

    // Close, then Pause or Resume and Clear finished (the footer runs right to left); the audio note in the space
    // left
    fn footer(&self, ui: &mut Ui) -> Option<RenderQueueAction> {
        let close = TextButton::new(ButtonKind::Secondary, Text::message(StudioUiMessage::Close)).show(ui).clicked();
        let (icon, word, toggle) = if self.view.paused {
            (Icon::Play, StudioUiMessage::Resume, RenderQueueAction::ResumeAll)
        } else {
            (Icon::Pause, StudioUiMessage::Pause, RenderQueueAction::PauseAll)
        };
        let pause =
            TextButton::new(ButtonKind::Ghost, Text::message(word)).leading(Leading::Icon(icon)).show(ui).clicked();
        let clear =
            TextButton::new(ButtonKind::Ghost, Text::message(StudioUiMessage::ClearFinished)).show(ui).clicked();
        Self::note(ui);
        if close {
            Some(RenderQueueAction::Close)
        } else if pause {
            Some(toggle)
        } else {
            clear.then_some(RenderQueueAction::ClearFinished)
        }
    }

    // Paper: the info mark (`--text-icon`) and "Video only · audio is not exported" (note, dim) at the footer's left
    fn note(ui: &mut Ui) {
        let rest = ui.available_rect_before_wrap();
        let dim = <color::TextDim as ColorToken>::VALUE;
        let icon = IconSize::Regular.points();
        let mark = Rect::from_min_size(pos2(rest.left(), rest.center().y - icon / 2.0), vec2(icon, icon));
        Icon::Info.paint(ui.painter(), mark, IconSize::Regular, dim);
        let at = pos2(mark.right() + Self::NOTE_GAP, rest.center().y);
        let room = (rest.right() - at.x).max(0.0);
        let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
        let note = Text::message(StudioUiMessage::AudioNote);
        Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &note, style, dim, TextFit::Truncate(room));
    }

    fn action(response: ModalResponse, action: Option<RenderQueueAction>) -> Option<RenderQueueAction> {
        if response.closed { Some(RenderQueueAction::Close) } else { action }
    }
}

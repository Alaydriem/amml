mod easing_family;
mod easing_popover_action;
mod easing_popover_view;
mod easing_preset;
mod handle_index;

pub use easing_family::EasingFamily;
pub use easing_popover_action::EasingPopoverAction;
pub use easing_popover_view::EasingPopoverView;
pub use easing_preset::EasingPreset;
pub use handle_index::HandleIndex;

use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::state::InteractionState;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{ButtonKind, IconButton, Segment, Segmented, TextButton};
use egui::{
    Align2, Color32, CursorIcon, Id, Painter, Pos2, Rect, ScrollArea, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder,
    pos2, vec2,
};

use super::{CurvePlot, CurvePlotView};
use crate::id::StudioUiId;
use crate::message::StudioUiMessage;

/// Paper: Menus + Dialogs, `Easing popover` (`7J6-0` custom bezier, `8MQ-0` named curve). The curve plot with its
/// handles (or, for a named curve, the button that makes it custom), the handle values, the saved presets, and the
/// preset grid under the family filter.
pub struct EasingPopover<'a> {
    view: &'a EasingPopoverView,
    id: Id,
}

impl<'a> EasingPopover<'a> {
    /// Paper: the card is 820 × 560.
    pub const WIDTH: f32 = 820.0;
    pub const HEIGHT: f32 = 560.0;
    // Paper: the header is 66 high with 22 each side and 14 on top; the subtitle 6 under the title
    const HEADER: f32 = 66.0;
    const SIDE: f32 = 22.0;
    const HEADER_TOP: f32 = 14.0;
    const SUBTITLE_GAP: f32 = 6.0;
    // Paper: the body is 16 down; the left column 250 wide, 22 from the right one, its parts 10 apart
    const BODY_TOP: f32 = 16.0;
    const COLUMN: f32 = 250.0;
    const COLUMN_GAP: f32 = 22.0;
    const GAP: f32 = 10.0;
    // Paper: the plot box is 250 square with the 190 square plot centred
    const PLOT: f32 = 250.0;
    const PLOT_INSET: f32 = 22.0;
    const HINT_BOTTOM: f32 = 4.0;
    // Paper: handle fields are 122 × 28, 6 apart; the make-custom button is 30 high
    const FIELD: [f32; 2] = [122.0, 28.0];
    const FIELD_GAP: f32 = 6.0;
    const FIELD_PADDING: f32 = 10.0;
    const BUTTON: f32 = 30.0;
    // Paper: the saved presets head is 8 lower; the save button 120 × 26; rows 240 × 30, 4 apart, with a 30 × 20 curve
    const SECTION_TOP: f32 = 8.0;
    const SAVE: [f32; 2] = [120.0, 26.0];
    const ROW: [f32; 2] = [240.0, 30.0];
    const ROW_GAP: f32 = 4.0;
    const ROW_PADDING: f32 = 6.0;
    const ICON: [f32; 2] = [30.0, 20.0];
    const ICON_STROKE: f32 = 1.4;
    // Paper: grid cards are 116 × 66, 8 apart, padded 6 and 12, with a 90 × 34 curve; the selected card has a 1.5 border
    const CARD: [f32; 2] = [116.0, 66.0];
    const CARD_GAP: f32 = 8.0;
    const CARD_PADDING: [f32; 2] = [12.0, 6.0];
    const CARD_CURVE: [f32; 2] = [90.0, 34.0];
    const CARD_STROKE: f32 = 1.6;
    const SELECTED_BORDER: f32 = 1.5;
    // a press this close to a handle takes it
    const HANDLE_HIT: f32 = 10.0;

    pub fn new(view: &'a EasingPopoverView) -> Self {
        Self { view, id: Id::unique(StudioUiId::EasingFamily) }
    }

    /// Another id for the family filter, for a second popover on one screen (the gallery shows two).
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn show(&self, ui: &mut Ui) -> Vec<EasingPopoverAction> {
        let (card, _) = ui.allocate_exact_size(vec2(Self::WIDTH, Self::HEIGHT), Sense::hover());
        let raised = <color::BgRaised as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        Draw::surface(ui.painter(), card, Corner::Panel, raised, Some(strong), Elevation::Floating);
        let mut actions = Vec::new();
        self.header(ui, card, &mut actions);
        self.left(ui, card, &mut actions);
        self.right(ui, card, &mut actions);
        // `allocate_rect` moved the cursor to each part; the page goes on under the card
        ui.advance_cursor_after_rect(card);
        actions
    }

    /// Where `handle` is drawn on a popover whose card is `card`; `None` for a named curve.
    pub fn handle_place(&self, card: Rect, handle: HandleIndex) -> Option<Pos2> {
        let [first, second] = self.view.handles?;
        let at = match handle {
            HandleIndex::First => first,
            HandleIndex::Second => second,
        };
        Some(CurvePlot::point(Self::plot_rect(card), at.x, at.y))
    }

    fn header(&self, ui: &mut Ui, card: Rect, actions: &mut Vec<EasingPopoverAction>) {
        let painter = ui.painter();
        let at = pos2(card.left() + Self::SIDE, card.top() + Self::HEADER_TOP);
        let title = Text::message(StudioUiMessage::Easing);
        let ink = <color::Text as ColorToken>::VALUE;
        let style = TextStyle::new(TypeSize::Title, FontFace::Semibold);
        let title = Draw::text(painter, at, Align2::LEFT_TOP, &title, style, ink, TextFit::Line);
        let side = <spacing::Row as SpacingToken>::VALUE;
        let close = Rect::from_center_size(
            pos2(card.right() - Self::SIDE - side / 2.0, card.top() + Self::HEADER / 2.0),
            vec2(side, side),
        );
        let room = (close.left() - Self::GAP - at.x).max(0.0);
        let sub = pos2(at.x, title.bottom() + Self::SUBTITLE_GAP);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let note = TextStyle::new(TypeSize::Note, FontFace::Regular);
        Draw::text(painter, sub, Align2::LEFT_TOP, &self.view.subtitle, note, dim, TextFit::Truncate(room));
        let header = Rect::from_min_size(card.min, vec2(card.width(), Self::HEADER));
        Draw::divider(painter, header.x_range(), header.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        let button = IconButton::new(Icon::Close).tooltip(Text::message(StudioUiMessage::Close));
        if button.show(&mut ui.new_child(UiBuilder::new().max_rect(close))).clicked() {
            actions.push(EasingPopoverAction::Close);
        }
    }

    fn plot_rect(card: Rect) -> Rect {
        let top = card.top() + Self::HEADER + Self::BODY_TOP;
        let plot = Rect::from_min_size(pos2(card.left() + Self::SIDE, top), vec2(Self::PLOT, Self::PLOT));
        plot.shrink(Self::PLOT_INSET)
    }

    // the plot, the handle fields or the make-custom button, and the saved presets
    fn left(&self, ui: &mut Ui, card: Rect, actions: &mut Vec<EasingPopoverAction>) {
        let top = card.top() + Self::HEADER + Self::BODY_TOP;
        let left = card.left() + Self::SIDE;
        let frame = Rect::from_min_size(pos2(left, top), vec2(Self::PLOT, Self::PLOT));
        let hue = self.view.track.color();
        for handle in [HandleIndex::First, HandleIndex::Second] {
            let Some(at) = self.handle_place(card, handle) else { break };
            let hit = Rect::from_center_size(at, egui::Vec2::splat(Self::HANDLE_HIT * 2.0));
            let response = ui.allocate_rect(hit, Sense::drag()).on_hover_cursor(CursorIcon::Grab);
            if let (true, Some(pointer)) = (response.dragged(), response.interact_pointer_pos()) {
                let plot = CurvePlot::plot_box(Self::plot_rect(card));
                let x = ((pointer.x - plot.left()) / plot.width()).clamp(0.0, 1.0);
                let y = (plot.bottom() - pointer.y) / plot.height();
                actions.push(EasingPopoverAction::MoveHandle(handle, pos2(x, y)));
            }
        }
        let painter = ui.painter();
        Draw::fill(painter, frame, Corner::Panel, <color::BgApp as ColorToken>::VALUE);
        Draw::stroke(painter, frame, Corner::Panel, <color::Line as ColorToken>::VALUE);
        let view = CurvePlotView { samples: &self.view.samples, color: hue, handles: self.view.handles };
        CurvePlot::new(&view).paint(painter, Self::plot_rect(card));
        let mut y = frame.bottom() + Self::GAP;
        if let Some([first, second]) = self.view.handles {
            let values = [
                (StudioUiMessage::HandleX1, first.x),
                (StudioUiMessage::HandleY1, first.y),
                (StudioUiMessage::HandleX2, second.x),
                (StudioUiMessage::HandleY2, second.y),
            ];
            for (index, (label, value)) in values.into_iter().enumerate() {
                let column = (index % 2) as f32;
                let row = (index / 2) as f32;
                let field = Rect::from_min_size(
                    pos2(
                        left + column * (Self::FIELD[0] + Self::FIELD_GAP),
                        y + row * (Self::FIELD[1] + Self::FIELD_GAP),
                    ),
                    vec2(Self::FIELD[0], Self::FIELD[1]),
                );
                Self::field(painter, field, label, value);
            }
            y += Self::FIELD[1] * 2.0 + Self::FIELD_GAP;
        } else {
            let hint = Text::message(StudioUiMessage::DragHandleHint);
            let micro = TextStyle::new(TypeSize::Micro, FontFace::Regular);
            let faint = <color::TextFaint as ColorToken>::VALUE;
            let at = pos2(frame.center().x, frame.bottom() - Self::HINT_BOTTOM);
            Draw::text(painter, at, Align2::CENTER_BOTTOM, &hint, micro, faint, TextFit::Line);
            let place = Rect::from_min_size(pos2(left, y), vec2(Self::COLUMN, Self::BUTTON));
            let button = TextButton::new(ButtonKind::Secondary, Text::message(StudioUiMessage::MakeCustom));
            if Self::button(ui, place, &button) {
                actions.push(EasingPopoverAction::MakeCustom);
            }
            y += Self::BUTTON;
        }
        y += Self::GAP + Self::SECTION_TOP;
        let save =
            Rect::from_min_size(pos2(left + Self::COLUMN - Self::SAVE[0], y), vec2(Self::SAVE[0], Self::SAVE[1]));
        let head = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let words = Text::message(StudioUiMessage::SavedPresets);
        Draw::text(ui.painter(), pos2(left, save.center().y), Align2::LEFT_CENTER, &words, head, faint, TextFit::Line);
        let button = TextButton::new(ButtonKind::Outline, Text::message(StudioUiMessage::SavePreset));
        if Self::button(ui, save, &button) {
            actions.push(EasingPopoverAction::SavePreset);
        }
        let list = Rect::from_min_max(
            pos2(left, save.bottom() + Self::GAP),
            pos2(left + Self::COLUMN, card.bottom() - Self::BODY_TOP),
        );
        let count = self.view.saved.len() as f32;
        let height = (count * (Self::ROW[1] + Self::ROW_GAP) - Self::ROW_GAP).max(0.0);
        let saved = self.id.with(StudioUiId::EasingSaved);
        let picked = Self::scroll(ui, list, saved, height, |ui, area| self.saved(ui, area, hue));
        actions.extend(picked);
    }

    // the saved preset rows from the top of `area`
    fn saved(&self, ui: &mut Ui, area: Rect, hue: Color32) -> Vec<EasingPopoverAction> {
        let mut actions = Vec::new();
        let left = area.left();
        let mut top = area.top();
        for (index, preset) in self.view.saved.iter().enumerate() {
            let row = Rect::from_min_size(pos2(left, top), vec2(Self::ROW[0], Self::ROW[1]));
            let response = ui.allocate_rect(row, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            let fill = if self.view.selected_saved == Some(index) {
                <color::AccentSoft as ColorToken>::VALUE
            } else if response.hovered() {
                <color::BgHover as ColorToken>::VALUE
            } else {
                <color::BgField as ColorToken>::VALUE
            };
            let painter = ui.painter();
            Draw::fill(painter, row, Corner::Control, fill);
            let icon = Rect::from_min_size(
                pos2(row.left() + Self::ROW_PADDING, row.center().y - Self::ICON[1] / 2.0),
                vec2(Self::ICON[0], Self::ICON[1]),
            );
            Self::curve(painter, icon, &preset.samples, hue, Self::ICON_STROKE);
            let x = icon.right() + Self::GAP;
            let note = TextStyle::new(TypeSize::Note, FontFace::Regular);
            let ink = <color::Text as ColorToken>::VALUE;
            let room = (row.right() - Self::ROW_PADDING - x).max(0.0);
            Draw::text(
                painter,
                pos2(x, row.center().y),
                Align2::LEFT_CENTER,
                &preset.name,
                note,
                ink,
                TextFit::Truncate(room),
            );
            if response.secondary_clicked() {
                if let Some(at) = response.interact_pointer_pos() {
                    actions.push(EasingPopoverAction::OpenSavedMenu(index, at));
                }
            } else if response.clicked() {
                actions.push(EasingPopoverAction::PickSaved(index));
            }
            top = row.bottom() + Self::ROW_GAP;
        }
        actions
    }

    // `body` in a vertical scroll area that fills `rect`, given content `height` points high at the area's top left
    fn scroll(
        ui: &mut Ui,
        rect: Rect,
        id: Id,
        height: f32,
        body: impl FnOnce(&mut Ui, Rect) -> Vec<EasingPopoverAction>,
    ) -> Vec<EasingPopoverAction> {
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
        ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .max_height(rect.height())
            .show(&mut child, |ui| {
                let (area, _) = ui.allocate_exact_size(vec2(rect.width(), height), Sense::hover());
                body(ui, area)
            })
            .inner
    }

    // the family filter and the preset grid
    fn right(&self, ui: &mut Ui, card: Rect, actions: &mut Vec<EasingPopoverAction>) {
        let left = card.left() + Self::SIDE + Self::COLUMN + Self::COLUMN_GAP;
        let top = card.top() + Self::HEADER + Self::BODY_TOP;
        let width = card.right() - Self::SIDE - left;
        let filter = Rect::from_min_size(pos2(left, top), vec2(width, Self::BUTTON));
        let mut family = self.view.family.index();
        let segments = EasingFamily::ALL.iter().map(|family| Segment::Text(family.label())).collect();
        let mut child = ui.new_child(UiBuilder::new().max_rect(filter));
        let changed = Segmented::new(self.id, &mut family, segments).show(&mut child, width).changed();
        if changed {
            actions.push(EasingPopoverAction::SetFamily(EasingFamily::ALL[family]));
        }
        let grid = Rect::from_min_max(
            pos2(left, filter.bottom() + Self::GAP),
            pos2(left + width, card.bottom() - Self::BODY_TOP),
        );
        let per_row = (((grid.width() + Self::CARD_GAP) / (Self::CARD[0] + Self::CARD_GAP)).floor() as usize).max(1);
        let shown: Vec<_> =
            self.view.presets.iter().enumerate().filter(|(_, preset)| self.view.family.shows(preset.family)).collect();
        let rows = shown.len().div_ceil(per_row) as f32;
        let height = (rows * (Self::CARD[1] + Self::CARD_GAP) - Self::CARD_GAP).max(0.0);
        let id = self.id.with(StudioUiId::EasingGrid);
        let picked = Self::scroll(ui, grid, id, height, |ui, area| {
            let mut actions = Vec::new();
            for (slot, (index, preset)) in shown.into_iter().enumerate() {
                let at = pos2(
                    area.left() + (slot % per_row) as f32 * (Self::CARD[0] + Self::CARD_GAP),
                    area.top() + (slot / per_row) as f32 * (Self::CARD[1] + Self::CARD_GAP),
                );
                let place = Rect::from_min_size(at, vec2(Self::CARD[0], Self::CARD[1]));
                let response = ui.allocate_rect(place, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
                Self::card(ui.painter(), place, preset, self.view.selected == Some(index), response.hovered());
                if response.clicked() {
                    actions.push(EasingPopoverAction::PickPreset(index));
                }
            }
            actions
        });
        actions.extend(picked);
    }

    fn card(painter: &Painter, place: Rect, preset: &EasingPreset, selected: bool, hovered: bool) {
        let (fill, line, ink) = if selected {
            (
                <color::AccentSoft as ColorToken>::VALUE,
                <color::AccentHover as ColorToken>::VALUE,
                <color::Text as ColorToken>::VALUE,
            )
        } else {
            let fill =
                if hovered { <color::BgHover as ColorToken>::VALUE } else { <color::BgField as ColorToken>::VALUE };
            (fill, <color::TextFaint as ColorToken>::VALUE, <color::TextDim as ColorToken>::VALUE)
        };
        Draw::fill(painter, place, Corner::Control, fill);
        if selected {
            let border = Stroke::new(Self::SELECTED_BORDER, <color::Accent as ColorToken>::VALUE);
            painter.rect_stroke(place, Corner::Control.radius(place), border, StrokeKind::Inside);
        }
        let [padding_x, padding_y] = Self::CARD_PADDING;
        let curve =
            Rect::from_min_size(place.min + vec2(padding_x, padding_y), vec2(Self::CARD_CURVE[0], Self::CARD_CURVE[1]));
        Self::curve(painter, curve, &preset.samples, line, Self::CARD_STROKE);
        let caption = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        let at = pos2(place.left() + padding_x, place.bottom() - padding_y);
        let room = (place.width() - padding_x * 2.0).max(0.0);
        Draw::text(painter, at, Align2::LEFT_BOTTOM, &preset.name, caption, ink, TextFit::Truncate(room));
    }

    // a handle value: its label at the left and its value at the right of a field box
    fn field(painter: &Painter, field: Rect, label: StudioUiMessage, value: f32) {
        Draw::fill(painter, field, Corner::Control, <color::BgField as ColorToken>::VALUE);
        let note = TextStyle::new(TypeSize::Note, FontFace::Regular);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let at = pos2(field.left() + Self::FIELD_PADDING, field.center().y);
        Draw::text(painter, at, Align2::LEFT_CENTER, &Text::message(label), note, faint, TextFit::Line);
        let ink = <color::Text as ColorToken>::VALUE;
        let galley = Draw::galley(painter, &format!("{value:.2}"), note, ink, TextFit::Line);
        let at = Align2::RIGHT_CENTER
            .anchor_size(pos2(field.right() - Self::FIELD_PADDING, field.center().y), galley.size());
        painter.galley(at.min, galley, ink);
    }

    // `button` in `place`; true when clicked
    fn button(ui: &mut Ui, place: Rect, button: &TextButton) -> bool {
        let response = ui.allocate_rect(place, Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        button.paint(ui.painter(), place, InteractionState::of(&response, true));
        response.clicked()
    }

    // `samples` as a line across `rect`, y up, kept inside it
    fn curve(painter: &Painter, rect: Rect, samples: &[f32], color: Color32, width: f32) {
        if samples.len() < 2 {
            return;
        }
        let last = (samples.len() - 1) as f32;
        let points = samples
            .iter()
            .enumerate()
            .map(|(index, y)| {
                let point = pos2(rect.left() + rect.width() * index as f32 / last, rect.bottom() - rect.height() * y);
                point.clamp(rect.min, rect.max)
            })
            .collect();
        painter.add(Shape::line(points, Stroke::new(width, color)));
    }
}

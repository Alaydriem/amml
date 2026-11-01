mod custom_control;
mod row_lines;
mod setting_control;

pub use custom_control::CustomControl;
pub use setting_control::SettingControl;

use amml_i18n::text::Text;
use egui::{Align, CursorIcon, Id, Layout, Painter, Rect, Response, Sense, Ui, UiBuilder, Vec2, pos2, vec2};

use self::row_lines::RowLines;
use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::motion::Motion;
use crate::state::{DisabledReason, InteractionState, Selection, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{Checkbox, Chip, Toggle};

/// Paper: Widgets, `Widget / Toggle`, "IN A SETTING ROW"; Mod manager, `Widget / Section card`. A
/// label and a control; for a toggle or a checkbox the whole row is the click area, and the row's
/// hover shows on the control. With help, a note, a chip or the changed marker, or in a section
/// card, the row grows with its wrapped text.
pub struct SettingRow<'a> {
    label: Text,
    control: SettingControl<'a>,
    disabled: Option<DisabledReason>,
    help: Option<Text>,
    changed: bool,
    note: Option<Text>,
    chip: Option<Chip>,
    card: bool,
}

impl<'a> SettingRow<'a> {
    /// Paper: Section card, the card's left and right padding.
    pub(crate) const CARD_PAD_X: f32 = 20.0;
    /// Paper: Section card, a row's top and bottom padding.
    const CARD_PAD_Y: f32 = 14.0;
    /// Paper: Section card, the changed marker's inset from the row's top and bottom.
    const MARKER_INSET: f32 = 10.0;
    const HELP: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Help, FontFace::Semibold);

    pub fn new(label: Text, control: SettingControl<'a>) -> Self {
        Self { label, control, disabled: None, help: None, changed: false, note: None, chip: None, card: false }
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// A line under the label in `--color-text-dim`. It wraps.
    pub fn help(mut self, text: Text) -> Self {
        self.help = Some(text);
        self
    }

    /// The accent marker at the row's left edge: the value has an edit that is not saved.
    pub fn changed(mut self, changed: bool) -> Self {
        self.changed = changed;
        self
    }

    /// A warning line with an icon under the help, such as "Changed in game to 2.50x".
    pub fn note(mut self, text: Text) -> Self {
        self.note = Some(text);
        self
    }

    /// A chip after the label's first line; Paper draws an Outline chip ("Applies after world
    /// reload").
    pub fn chip(mut self, chip: Chip) -> Self {
        self.chip = Some(chip);
        self
    }

    /// The section card look: the label in `--text-body`, the card's padding, no row fill.
    pub(crate) fn in_card(mut self) -> Self {
        self.card = true;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        if self.grows() {
            return self.show_grown(ui);
        }
        let enabled = self.disabled.is_none();
        // an `Other` control's width is known only after it is shown: use the last frame's
        let width_id = ui.next_auto_id().with("control width");
        let control_width = self.control_size(ui, width_id).x;
        let size = vec2(ui.available_width(), <spacing::Field as SpacingToken>::VALUE);
        let (rect, mut response) = ui.allocate_exact_size(size, self.sense());
        let state = InteractionState::of(&response, enabled);
        let painter = state.painter(ui.painter());
        let Self { label, control, disabled, .. } = self;
        Self::paint_row(&painter, rect, state, &label, control_width);
        let area = rect.shrink2(vec2(<spacing::S as SpacingToken>::VALUE, 0.0));
        let used = Self::run_control(ui, control, &mut response, (&painter, state), area, enabled);
        if let Some(used) = used {
            let last = ui.ctx().data(|data| data.get_temp::<f32>(width_id));
            if last.is_none_or(|last| (last - used).abs() > 0.5) {
                ui.ctx().data_mut(|data| data.insert_temp(width_id, used));
                ui.ctx().request_discard("a setting row's control changed width");
            }
        }
        Self::finish(response, disabled.as_ref())
    }

    /// Paints the row in `state`, for the gallery. An `Other` control is not painted: it needs a `Ui`.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState) {
        let painter = state.painter(painter);
        let control_width = match &self.control {
            SettingControl::Toggle(_) => Toggle::TRACK.x,
            SettingControl::Checkbox(_) => <spacing::L as SpacingToken>::VALUE,
            SettingControl::Other(_) => 0.0,
        };
        Self::paint_row(&painter, rect, state, &self.label, control_width);
        let area = rect.shrink2(vec2(<spacing::S as SpacingToken>::VALUE, 0.0));
        match &self.control {
            SettingControl::Toggle(on) => {
                let amount = if **on { 1.0 } else { 0.0 };
                Toggle::paint_amount(&painter, Self::control_rect(area, Toggle::TRACK), state.inner(), amount);
            }
            SettingControl::Checkbox(value) => {
                let mut value = **value;
                Checkbox::new(&mut value).paint(&painter, Self::box_rect(area), state.inner());
            }
            SettingControl::Other(_) => {}
        }
    }

    fn grows(&self) -> bool {
        self.card || self.help.is_some() || self.note.is_some() || self.chip.is_some() || self.changed
    }

    fn sense(&self) -> Sense {
        let row_clicks = !matches!(self.control, SettingControl::Other(_));
        if self.disabled.is_none() && row_clicks { Sense::click() } else { Sense::hover() }
    }

    // the row with its text column laid out first, so it is as high as its wrapped text
    fn show_grown(mut self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let (pad_x, pad_y) = if self.card {
            (Self::CARD_PAD_X, Self::CARD_PAD_Y)
        } else {
            (<spacing::S as SpacingToken>::VALUE, <spacing::S as SpacingToken>::VALUE)
        };
        let width = ui.available_width();
        // an `Other` control's width is known only after it is shown: use the last frame's
        let width_id = ui.next_auto_id().with("control width");
        let control = self.control_size(ui, width_id);
        let gap = if control.x > 0.0 { <spacing::L as SpacingToken>::VALUE } else { 0.0 };
        let column = (width - 2.0 * pad_x - control.x - gap).max(0.0);
        let chip = self.chip.take();
        let lines = self.lines(ui, column, chip);
        let height = lines.height().max(control.y) + 2.0 * pad_y;
        let height = if self.card { height } else { height.max(<spacing::Field as SpacingToken>::VALUE) };
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), self.sense());
        let state = InteractionState::of(&response, enabled);
        let painter = state.painter(ui.painter());
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Control.radius(rect));
        }
        if !self.card {
            Draw::fill(&painter, rect, Corner::Control, StateStyle::Clear.fill(state));
        }
        if self.changed {
            let marker = Rect::from_min_max(
                pos2(rect.left(), rect.top() + Self::MARKER_INSET),
                pos2(rect.left() + <spacing::Indicator as SpacingToken>::VALUE, rect.bottom() - Self::MARKER_INSET),
            );
            Draw::fill(&painter, marker, Corner::Full, <color::Accent as ColorToken>::VALUE);
        }
        let top = rect.center().y - lines.height() / 2.0;
        lines.paint(&painter, pos2(rect.left() + pad_x, top));

        let Self { control, disabled, .. } = self;
        let area = rect.shrink2(vec2(pad_x, 0.0));
        let used = Self::run_control(ui, control, &mut response, (&painter, state), area, enabled);
        if let Some(used) = used {
            let last = ui.ctx().data(|data| data.get_temp::<f32>(width_id));
            if last.is_none_or(|last| (last - used).abs() > 0.5) {
                ui.ctx().data_mut(|data| data.insert_temp(width_id, used));
                ui.ctx().request_discard("a setting row's control changed width");
            }
        }
        Self::finish(response, disabled.as_ref())
    }

    // the control's size before it is shown; an `Other` control's width from the last frame
    fn control_size(&self, ui: &Ui, width_id: Id) -> Vec2 {
        match self.control {
            SettingControl::Toggle(_) => Toggle::TRACK,
            SettingControl::Checkbox(_) => Vec2::splat(<spacing::L as SpacingToken>::VALUE),
            SettingControl::Other(_) => vec2(ui.ctx().data(|data| data.get_temp::<f32>(width_id)).unwrap_or(0.0), 0.0),
        }
    }

    // the label (with `chip` after it), help and note, wrapped at `width`
    fn lines(&self, ui: &Ui, width: f32, chip: Option<Chip>) -> RowLines {
        let painter = ui.painter();
        let ctx = ui.ctx();
        let text = <color::Text as ColorToken>::VALUE;
        let size = if self.card { TypeSize::Body } else { TypeSize::Label };
        let chip_width =
            chip.as_ref().map_or(0.0, |chip| chip.desired_width(ctx) + <spacing::M as SpacingToken>::VALUE);
        let label = UiText::resolve(ctx, &self.label);
        let style = TextStyle::new(size, FontFace::Regular);
        // the chip stays beside the label only while the label then fits on one line
        let chip_below = chip.is_some() && Draw::measure(ctx, &label, style).x + chip_width > width;
        let label_width = if chip_below { width } else { (width - chip_width).max(0.0) };
        let label = Draw::galley(painter, &label, style, text, TextFit::Wrap(label_width));
        let dim = <color::TextDim as ColorToken>::VALUE;
        let help = self
            .help
            .as_ref()
            .map(|help| Draw::galley(painter, &UiText::resolve(ctx, help), Self::HELP, dim, TextFit::Wrap(width)));
        let warning = <color::Warning as ColorToken>::VALUE;
        let note_width = (width - RowLines::note_indent()).max(0.0);
        let note = self.note.as_ref().map(|note| {
            Draw::galley(painter, &UiText::resolve(ctx, note), Self::NOTE, warning, TextFit::Wrap(note_width))
        });
        RowLines { label, chip, chip_below, width, help, note }
    }

    // draws or runs the control in `area`; returns an `Other` control's width
    fn run_control(
        ui: &mut Ui,
        control: SettingControl<'_>,
        response: &mut Response,
        (painter, state): (&Painter, InteractionState),
        area: Rect,
        enabled: bool,
    ) -> Option<f32> {
        match control {
            SettingControl::Toggle(on) => {
                if response.clicked() {
                    *on = !*on;
                    response.mark_changed();
                }
                let amount = Motion::flag(ui.ctx(), response.id, *on);
                Toggle::paint_amount(painter, Self::control_rect(area, Toggle::TRACK), state.inner(), amount);
                None
            }
            SettingControl::Checkbox(value) => {
                if response.clicked() {
                    *value = if *value == Selection::On { Selection::Off } else { Selection::On };
                    response.mark_changed();
                }
                Checkbox::new(value).paint(painter, Self::box_rect(area), state.inner());
                None
            }
            SettingControl::Other(control) => {
                let mut child =
                    ui.new_child(UiBuilder::new().max_rect(area).layout(Layout::right_to_left(Align::Center)));
                if !enabled {
                    child.disable();
                }
                *response |= control(&mut child, state);
                Some(child.min_rect().width())
            }
        }
    }

    fn finish(response: Response, disabled: Option<&DisabledReason>) -> Response {
        let clickable = response.sense.senses_click();
        match disabled {
            None if clickable => response.on_hover_cursor(CursorIcon::PointingHand),
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    // the row itself, on a painter that already carries the disabled opacity
    // the label may use the row less its padding, the control and the gap to the control
    fn paint_row(painter: &Painter, rect: Rect, state: InteractionState, label: &Text, control_width: f32) {
        if state == InteractionState::Focused {
            FocusRing::paint(painter, rect, Corner::Control.radius(rect));
        }
        Draw::fill(painter, rect, Corner::Control, StateStyle::Clear.fill(state));
        let text = UiText::resolve(painter.ctx(), label);
        let style = TextStyle::new(TypeSize::Label, FontFace::Regular);
        let color = <color::Text as ColorToken>::VALUE;
        let padding = <spacing::S as SpacingToken>::VALUE;
        let gap = if control_width > 0.0 { <spacing::L as SpacingToken>::VALUE } else { 0.0 };
        let room = (rect.width() - 2.0 * padding - control_width - gap).max(0.0);
        let galley = Draw::galley(painter, &text, style, color, TextFit::Truncate(room));
        painter.galley(pos2(rect.left() + padding, rect.center().y - galley.size().y / 2.0), galley, color);
    }

    fn box_rect(area: Rect) -> Rect {
        let side = <spacing::L as SpacingToken>::VALUE;
        Self::control_rect(area, vec2(side, side))
    }

    // the control's rect: `size`, at the right edge of `area`, centred on the height
    fn control_rect(area: Rect, size: Vec2) -> Rect {
        Rect::from_min_size(pos2(area.right() - size.x, area.center().y - size.y / 2.0), size)
    }
}

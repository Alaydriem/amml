mod dropdown_option;

pub use dropdown_option::DropdownOption;

use amml_i18n::text::Text;
use egui::{
    Color32, CursorIcon, Frame, Id, Key, Painter, Popup, PopupCloseBehavior, Pos2, Rect, RectAlign, Response, Sense,
    Ui, Vec2, pos2, vec2,
};

use crate::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::icon::{Icon, IconSize};
use crate::id::AmmlUiId;
use crate::state::{ArrowKeys, DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::{Elevation, FocusRing};
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Dropdown`. A field that opens a list of options below it (or above,
/// when there is no room). Up and Down move, Enter picks, Escape closes.
pub struct Dropdown<'a> {
    id: Id,
    selected: &'a mut Option<usize>,
    options: Vec<DropdownOption>,
    placeholder: Option<Text>,
    disabled: Option<DisabledReason>,
}

impl<'a> Dropdown<'a> {
    const VALUE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const SELECTED: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Semibold);
    const ALIGNS: [RectAlign; 1] = [RectAlign::TOP_START];

    pub fn new(id: Id, selected: &'a mut Option<usize>, options: Vec<DropdownOption>) -> Self {
        Self { id, selected, options, placeholder: None, disabled: None }
    }

    /// Shown in `--color-text-faint` while nothing is selected.
    pub fn placeholder(mut self, text: Text) -> Self {
        self.placeholder = Some(text);
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// Shows the field `width` wide. The response is changed only when an option is picked.
    pub fn show(self, ui: &mut Ui, width: f32) -> Response {
        let enabled = self.disabled.is_none();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, _) = ui.allocate_exact_size(vec2(width, <spacing::Field as SpacingToken>::VALUE), Sense::hover());
        let mut response = ui.interact(rect, self.id, sense);
        let popup_id = Popup::default_response_id(&response);
        let open = enabled && Popup::is_id_open(ui.ctx(), popup_id);
        let state = InteractionState::of(&response, enabled);
        self.paint(ui.painter(), rect, state, open);
        let highlight_id = self.id.with(AmmlUiId::DropdownHighlight);
        let mut picked = None;
        if open {
            ArrowKeys::keep(ui, response.id);
            let mut highlight = ui.data(|data| data.get_temp::<usize>(highlight_id)).or(*self.selected).unwrap_or(0);
            let (down, up, enter, escape) = ui.input(|input| {
                (
                    input.key_pressed(Key::ArrowDown),
                    input.key_pressed(Key::ArrowUp),
                    input.key_pressed(Key::Enter),
                    input.key_pressed(Key::Escape),
                )
            });
            if down {
                highlight = self.next_enabled(highlight, 1);
            }
            if up {
                highlight = self.next_enabled(highlight, -1);
            }
            ui.data_mut(|data| data.insert_temp(highlight_id, highlight));
            if enter && self.options.get(highlight).is_some_and(|option| option.enabled) {
                picked = Some(highlight);
            }
            if escape || picked.is_some() {
                Popup::close_id(ui.ctx(), popup_id);
            }
        } else {
            ui.data_mut(|data| data.remove::<usize>(highlight_id));
        }
        let highlight = ui.data(|data| data.get_temp::<usize>(highlight_id));
        if enabled && picked.is_none() {
            Popup::from_toggle_button_response(&response)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .frame(Self::list_frame())
                .gap(<spacing::Xs as SpacingToken>::VALUE)
                .width(rect.width() - Self::list_padding() * 2.0)
                .align(RectAlign::BOTTOM_START)
                .align_alternatives(&Self::ALIGNS)
                .show(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    for (index, option) in self.options.iter().enumerate() {
                        let sense = if option.enabled { Sense::click() } else { Sense::hover() };
                        let row = vec2(ui.available_width(), <spacing::Row as SpacingToken>::VALUE);
                        let (row_rect, row_response) = ui.allocate_exact_size(row, sense);
                        let hovered = row_response.hovered() || highlight == Some(index);
                        self.paint_option(ui.painter(), row_rect, index, hovered);
                        if row_response.clicked() {
                            picked = Some(index);
                            ui.close();
                        }
                    }
                });
        }
        if let Some(index) = picked
            && *self.selected != Some(index)
        {
            *self.selected = Some(index);
            response.mark_changed();
        }
        match &self.disabled {
            None => response.on_hover_cursor(CursorIcon::PointingHand),
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints the field in `state`, open or closed, for the gallery.
    pub fn paint(&self, painter: &Painter, rect: Rect, state: InteractionState, open: bool) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Control.radius(rect));
        }
        let fill_state = if open { InteractionState::Pressed } else { state.inner() };
        Draw::fill(&painter, rect, Corner::Control, StateStyle::Field.fill(fill_state));
        Draw::stroke(&painter, rect, Corner::Control, <color::LineStrong as ColorToken>::VALUE);
        let active = open || matches!(state, InteractionState::Hover | InteractionState::Pressed);
        let padding = <spacing::M as SpacingToken>::VALUE;
        let small = IconSize::Small.points();
        let chevron =
            Rect::from_center_size(pos2(rect.right() - padding - small / 2.0, rect.center().y), vec2(small, small));
        let chevron_colour =
            if active { <color::Text as ColorToken>::VALUE } else { <color::TextDim as ColorToken>::VALUE };
        let icon = if open { Icon::ChevronUp } else { Icon::ChevronDown };
        icon.paint(&painter, chevron, IconSize::Small, chevron_colour);
        let (text, colour) = match self.selected.and_then(|index| self.options.get(index)) {
            Some(option) => (UiText::resolve(painter.ctx(), &option.text), <color::Text as ColorToken>::VALUE),
            None => {
                let placeholder = self.placeholder.as_ref().map(|text| UiText::resolve(painter.ctx(), text));
                let colour = if active {
                    <color::TextDim as ColorToken>::VALUE
                } else {
                    <color::TextFaint as ColorToken>::VALUE
                };
                (placeholder.unwrap_or_default(), colour)
            }
        };
        let room = (chevron.left() - padding - rect.left() - padding).max(0.0);
        let galley = Draw::galley(&painter, &text, Self::VALUE, colour, TextFit::Truncate(room));
        painter.galley(pos2(rect.left() + padding, rect.center().y - galley.size().y / 2.0), galley, colour);
    }

    /// Paints the open list with its top-left at `top_left`, `width` wide, `hovered` row hovered, for
    /// the gallery. Returns its rect.
    pub fn paint_list(&self, painter: &Painter, top_left: Pos2, width: f32, hovered: Option<usize>) -> Rect {
        let padding = Self::list_padding();
        let row = <spacing::Row as SpacingToken>::VALUE;
        let rect = Rect::from_min_size(top_left, vec2(width, row * self.options.len() as f32 + padding * 2.0));
        let raised = <color::BgRaised as ColorToken>::VALUE;
        let strong = <color::LineStrong as ColorToken>::VALUE;
        Draw::surface(painter, rect, Corner::Control, raised, Some(strong), Elevation::Floating);
        for index in 0..self.options.len() {
            let top = rect.top() + padding + row * index as f32;
            let row_rect = Rect::from_min_size(pos2(rect.left() + padding, top), vec2(width - padding * 2.0, row));
            self.paint_option(painter, row_rect, index, hovered == Some(index));
        }
        rect
    }

    fn paint_option(&self, painter: &Painter, rect: Rect, index: usize, hovered: bool) {
        let Some(option) = self.options.get(index) else { return };
        let painter = if option.enabled { painter.clone() } else { InteractionState::Disabled.painter(painter) };
        let selected = *self.selected == Some(index);
        let fill = if selected {
            <color::AccentSoft as ColorToken>::VALUE
        } else if hovered && option.enabled {
            <color::BgHover as ColorToken>::VALUE
        } else {
            Color32::TRANSPARENT
        };
        Draw::fill(&painter, rect, Corner::Small, fill);
        let padding = <spacing::S as SpacingToken>::VALUE;
        let (style, colour) = if selected {
            (Self::SELECTED, <color::AccentHover as ColorToken>::VALUE)
        } else {
            (Self::VALUE, <color::Text as ColorToken>::VALUE)
        };
        let small = IconSize::Small.points();
        if selected {
            let check =
                Rect::from_center_size(pos2(rect.right() - padding - small / 2.0, rect.center().y), vec2(small, small));
            Icon::Check.paint(&painter, check, IconSize::Small, colour);
        }
        let text = UiText::resolve(painter.ctx(), &option.text);
        let room = (rect.width() - padding * 3.0 - small).max(0.0);
        let galley = Draw::galley(&painter, &text, style, colour, TextFit::Truncate(room));
        painter.galley(pos2(rect.left() + padding, rect.center().y - galley.size().y / 2.0), galley, colour);
    }

    // the next enabled option from `from` in `direction`, or `from` when there is none
    fn next_enabled(&self, from: usize, direction: isize) -> usize {
        let mut index = from as isize;
        loop {
            index += direction;
            match usize::try_from(index).ok().and_then(|at| self.options.get(at).map(|option| (at, option))) {
                Some((at, option)) if option.enabled => return at,
                Some(_) => {}
                None => return from,
            }
        }
    }

    // the list panel: `--color-bg-raised`, 1 px `--color-line-strong`, `--radius-control`, Floating
    fn list_frame() -> Frame {
        let padding = Self::list_padding();
        Frame::new()
            .fill(<color::BgRaised as ColorToken>::VALUE)
            .stroke(Draw::border(<color::LineStrong as ColorToken>::VALUE))
            .corner_radius(Corner::Control.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(padding, padding))
            .shadow(Elevation::Floating.shadow())
    }

    fn list_padding() -> f32 {
        <spacing::Xs as SpacingToken>::VALUE
    }
}

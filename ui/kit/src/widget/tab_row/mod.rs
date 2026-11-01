mod tab_row_size;

pub use tab_row_size::TabRowSize;

use amml_i18n::text::Text;
use egui::{Context, CursorIcon, Id, Key, Painter, Rect, Response, Sense, Ui, pos2, vec2};

use crate::draw::{Corner, Draw, TextFit};
use crate::icon::{Icon, IconSize};
use crate::id::AmmlUiId;
use crate::motion::Motion;
use crate::state::{ArrowKeys, DisabledReason, InteractionState, StateStyle};
use crate::text::UiText;
use crate::tokens::rule::FocusRing;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};

/// Paper: Widgets, `Widget / Tab row`. Tabs on a strip with a 1 px divider; the selected tab has
/// `--color-text` and a 2 px accent underline that slides.
pub struct TabRow<'a> {
    id: Id,
    selected: &'a mut usize,
    tabs: Vec<Text>,
    size: TabRowSize,
    icons: Vec<Icon>,
    disabled: Option<DisabledReason>,
}

impl<'a> TabRow<'a> {
    pub fn new(id: Id, selected: &'a mut usize, tabs: Vec<Text>) -> Self {
        Self { id, selected, tabs, size: TabRowSize::Inspector, icons: Vec::new(), disabled: None }
    }

    pub fn size(mut self, size: TabRowSize) -> Self {
        self.size = size;
        self
    }

    /// An icon before each label, in tab order (Paper: Mod manager screen 01). The selected tab's
    /// icon is `--color-accent-hover`, the others take their label's colour.
    pub fn icons(mut self, icons: Vec<Icon>) -> Self {
        self.icons = icons;
        self
    }

    pub fn enabled(mut self, enabled: bool, reason: DisabledReason) -> Self {
        self.disabled = if enabled { None } else { Some(reason) };
        self
    }

    /// The width one tab takes: its label plus the size's padding on each side.
    pub fn tab_width(ctx: &Context, text: &Text, size: TabRowSize) -> f32 {
        (Draw::measure(ctx, &UiText::resolve(ctx, text), size.text()).x + size.padding() * 2.0).ceil()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let enabled = self.disabled.is_none();
        let height = self.size.height() + <spacing::Divider as SpacingToken>::VALUE;
        let (rect, mut response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let tabs = self.tab_rects(ui.ctx(), rect);
        let mut states = Vec::with_capacity(tabs.len());
        let mut focused = None;
        for (index, tab) in tabs.iter().enumerate() {
            let sense = if enabled { Sense::click() } else { Sense::hover() };
            let tab_response = ui.interact(*tab, self.id.with(index), sense);
            if tab_response.clicked() && *self.selected != index {
                *self.selected = index;
                response.mark_changed();
            }
            if tab_response.has_focus() {
                ArrowKeys::keep(ui, tab_response.id);
                focused = Some(index);
            }
            states.push(InteractionState::of(&tab_response, enabled));
            if enabled {
                response |= tab_response.on_hover_cursor(CursorIcon::PointingHand);
            }
        }
        if let Some(index) = focused {
            let step = ui.input(|input| {
                i32::from(input.key_pressed(Key::ArrowRight)) - i32::from(input.key_pressed(Key::ArrowLeft))
            });
            let target = (index as i64 + i64::from(step)).clamp(0, tabs.len() as i64 - 1) as usize;
            if step != 0 && target != *self.selected {
                *self.selected = target;
                response.mark_changed();
                ui.memory_mut(|memory| memory.request_focus(self.id.with(target)));
            }
        }
        let painter = if enabled { ui.painter().clone() } else { InteractionState::Disabled.painter(ui.painter()) };
        Draw::divider(&painter, rect.x_range(), rect.bottom() - 1.0, <color::Line as ColorToken>::VALUE);
        for (index, (tab, text)) in tabs.iter().zip(&self.tabs).enumerate() {
            if states[index] == InteractionState::Focused {
                FocusRing::paint(&painter, *tab, Corner::Control.radius(*tab));
                Draw::fill(&painter, *tab, Corner::Control, <color::BgPanel as ColorToken>::VALUE);
            }
            let icon = self.icons.get(index).copied();
            let selected = index == *self.selected;
            Self::paint_body(&painter, *tab, (text, icon), self.size, states[index].inner(), selected);
        }
        if let Some(tab) = tabs.get(*self.selected) {
            let target = Self::underline(*tab, self.size);
            let left = Motion::value(ui.ctx(), self.id.with(AmmlUiId::UnderlineLeft), target.left());
            let right = Motion::value(ui.ctx(), self.id.with(AmmlUiId::UnderlineRight), target.right());
            let rect = Rect::from_min_max(pos2(left, target.top()), pos2(right, target.bottom()));
            Draw::fill(&painter, rect, Corner::Full, <color::Accent as ColorToken>::VALUE);
        }
        match &self.disabled {
            None => response,
            Some(reason) => reason.tooltip().attach(response),
        }
    }

    /// Paints one tab in `state` into `rect`, with its underline when `selected`, for the gallery.
    pub fn paint_tab(
        painter: &Painter,
        rect: Rect,
        text: &Text,
        size: TabRowSize,
        state: InteractionState,
        selected: bool,
    ) {
        let painter = state.painter(painter);
        if state == InteractionState::Focused {
            FocusRing::paint(&painter, rect, Corner::Control.radius(rect));
            Draw::fill(&painter, rect, Corner::Control, <color::BgPanel as ColorToken>::VALUE);
        }
        Self::paint_body(&painter, rect, (text, None), size, state.inner(), selected);
        if selected {
            Draw::fill(&painter, Self::underline(rect, size), Corner::Full, <color::Accent as ColorToken>::VALUE);
        }
    }

    fn paint_body(
        painter: &Painter,
        rect: Rect,
        (text, icon): (&Text, Option<Icon>),
        size: TabRowSize,
        state: InteractionState,
        selected: bool,
    ) {
        let fill = if selected { egui::Color32::TRANSPARENT } else { StateStyle::Clear.fill(state) };
        Draw::fill(painter, rect, Corner::Control, fill);
        let color = if selected { <color::Text as ColorToken>::VALUE } else { <color::TextFaint as ColorToken>::VALUE };
        let label = UiText::resolve(painter.ctx(), text);
        let lead = icon.map_or(0.0, |_| Self::icon_room());
        let room = (rect.width() - size.padding() * 2.0 - lead).max(0.0);
        let galley = Draw::galley(painter, &label, size.text(), color, TextFit::Truncate(room));
        // the icon and the label are centred together
        let left = rect.center().x - (lead + galley.size().x) / 2.0;
        if let Some(icon) = icon {
            let side = IconSize::Regular.points();
            let slot = Rect::from_min_size(pos2(left, rect.center().y - side / 2.0), vec2(side, side));
            let tint = if selected { <color::AccentHover as ColorToken>::VALUE } else { color };
            icon.paint(painter, slot, IconSize::Regular, tint);
        }
        painter.galley(pos2(left + lead, rect.center().y - galley.size().y / 2.0), galley, color);
    }

    // an icon and the `--spacing-s` gap after it
    fn icon_room() -> f32 {
        IconSize::Regular.points() + <spacing::S as SpacingToken>::VALUE
    }

    // the 2 px underline at the bottom of a tab, inset by the tab padding
    fn underline(tab: Rect, size: TabRowSize) -> Rect {
        let thickness = <spacing::BorderStrong as SpacingToken>::VALUE;
        Rect::from_min_max(
            pos2(tab.left() + size.padding(), tab.bottom() - thickness),
            pos2(tab.right() - size.padding(), tab.bottom()),
        )
    }

    // tabs laid left to right from the strip's `--spacing-s` padding, above the divider
    fn tab_rects(&self, ctx: &Context, rect: Rect) -> Vec<Rect> {
        let mut x = rect.left() + <spacing::S as SpacingToken>::VALUE;
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let lead = if self.icons.get(index).is_some() { Self::icon_room() } else { 0.0 };
                let width = Self::tab_width(ctx, text, self.size) + lead;
                let tab = Rect::from_min_size(pos2(x, rect.top()), vec2(width, self.size.height()));
                x += width;
                tab
            })
            .collect()
    }
}

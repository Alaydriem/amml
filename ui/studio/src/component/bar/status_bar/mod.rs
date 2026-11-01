mod status_bar_layout;
mod status_bar_view;

pub use status_bar_layout::StatusBarLayout;
pub use status_bar_view::StatusBarView;

use amml_i18n::text::Text;
use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use egui::{Align2, Pos2, Rect, Response, Sense, Ui, pos2, vec2};

use crate::message::StudioUiMessage;
use crate::tokens::color as studio;
use crate::tokens::selector::StatusColor;

/// Paper: Editor components, `Status bar` (`20D-0`). The mode dot and words, the detail, then the key hints and
/// the coverage legend on the right.
pub struct StatusBar<'a> {
    view: &'a StatusBarView,
    layout: StatusBarLayout,
}

impl<'a> StatusBar<'a> {
    // Paper `20D-0`: padding 12 left and 16 right; a 7 px mode dot with the words 10 after it
    const PADDING_LEFT: f32 = 12.0;
    const PADDING_RIGHT: f32 = 16.0;
    const MODE_DOT: f32 = 7.0;
    const MODE_GAP: f32 = 10.0;
    // Paper: the detail and the hints are set off by a middle dot with 3 spaces each side; code draws a 2 px dot
    // with this gap each side
    const SEPARATOR_GAP: f32 = 12.0;
    const SEPARATOR_DOT: f32 = 1.0;
    // Paper: legend items are 9 apart, each an 8 px dot 5 from its word; the hints end 16 before the legend
    const LEGEND_GAP: f32 = 9.0;
    const LEGEND_DOT: f32 = 8.0;
    const LEGEND_DOT_GAP: f32 = 5.0;
    const HINTS_GAP: f32 = 16.0;
    const MODE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Semibold);
    const DETAIL: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const HINTS: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    const LEGEND: TextStyle = TextStyle::new(TypeSize::Micro, FontFace::Semibold);

    pub fn new(view: &'a StatusBarView) -> Self {
        Self { view, layout: StatusBarLayout::Full }
    }

    pub fn layout(mut self, layout: StatusBarLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        let height = <spacing::StatusBar as SpacingToken>::VALUE;
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0, <color::BgApp as ColorToken>::VALUE);
        if self.view.tinted {
            painter.rect_filled(rect, 0, <studio::ModePreviewSoft as ColorToken>::VALUE);
        }
        Draw::divider(painter, rect.x_range(), rect.top() + 0.5, <color::Line as ColorToken>::VALUE);
        let legend_left = Self::legend(ui, rect);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let hints = UiText::resolve(ui.ctx(), &self.view.shortcuts);
        let hints = Draw::galley(painter, &hints, Self::HINTS, dim, TextFit::Line);
        let hints_rect =
            Align2::RIGHT_CENTER.anchor_size(pos2(legend_left - Self::HINTS_GAP, rect.center().y), hints.size());
        let x = self.mode(ui, rect);
        if hints_rect.left() > x {
            painter.galley(hints_rect.min, hints, dim);
        }
        if self.layout == StatusBarLayout::Full {
            let room = (hints_rect.left() - Self::SEPARATOR_GAP - x - Self::SEPARATOR_GAP * 2.0).max(0.0);
            Self::separator(ui, pos2(x + Self::SEPARATOR_GAP, rect.center().y));
            let detail = UiText::resolve(ui.ctx(), &self.view.detail);
            let detail = Draw::galley(painter, &detail, Self::DETAIL, dim, TextFit::Truncate(room));
            let at =
                Align2::LEFT_CENTER.anchor_size(pos2(x + Self::SEPARATOR_GAP * 2.0, rect.center().y), detail.size());
            painter.galley(at.min, detail, dim);
        }
        response
    }

    // the mode dot and words; returns their right edge
    fn mode(&self, ui: &Ui, rect: Rect) -> f32 {
        let painter = ui.painter();
        let x = rect.left() + Self::PADDING_LEFT;
        painter.circle_filled(
            pos2(x + Self::MODE_DOT / 2.0, rect.center().y),
            Self::MODE_DOT / 2.0,
            self.view.mode.color(),
        );
        let at = pos2(x + Self::MODE_DOT + Self::MODE_GAP, rect.center().y);
        let ink = <color::Text as ColorToken>::VALUE;
        Draw::text(painter, at, Align2::LEFT_CENTER, &self.view.mode_text, Self::MODE, ink, TextFit::Line).right()
    }

    fn separator(ui: &Ui, centre: Pos2) {
        ui.painter().circle_filled(centre, Self::SEPARATOR_DOT, <color::TextFaint as ColorToken>::VALUE);
    }

    // the five coverage statuses, ending at the right padding; returns their left edge
    fn legend(ui: &Ui, rect: Rect) -> f32 {
        let painter = ui.painter();
        let items = [
            (StatusColor::Works, StudioUiMessage::LegendWorks),
            (StatusColor::Engine, StudioUiMessage::LegendEngine),
            (StatusColor::Planned, StudioUiMessage::LegendPlanned),
            (StatusColor::Mockup, StudioUiMessage::LegendMockup),
            (StatusColor::Missing, StudioUiMessage::LegendMissing),
        ];
        let mut right = rect.right() - Self::PADDING_RIGHT;
        for (status, word) in items.into_iter().rev() {
            let hue = status.color();
            let label = Draw::text(
                painter,
                pos2(right, rect.center().y),
                Align2::RIGHT_CENTER,
                &Text::message(word),
                Self::LEGEND,
                hue,
                TextFit::Line,
            );
            let dot = Rect::from_center_size(
                pos2(label.left() - Self::LEGEND_DOT_GAP - Self::LEGEND_DOT / 2.0, rect.center().y),
                vec2(Self::LEGEND_DOT, Self::LEGEND_DOT),
            );
            painter.circle_filled(dot.center(), Self::LEGEND_DOT / 2.0, hue);
            right = dot.left() - Self::LEGEND_GAP;
        }
        right + Self::LEGEND_GAP
    }
}

mod coverage_filter;
mod coverage_group;
mod coverage_panel_action;
mod coverage_panel_view;
mod coverage_row;

pub use coverage_filter::CoverageFilter;
pub use coverage_group::CoverageGroup;
pub use coverage_panel_action::CoveragePanelAction;
pub use coverage_panel_view::CoveragePanelView;
pub use coverage_row::CoverageRow;

use amml_i18n::text::{Args, Text};
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::text::UiText;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{EmptyState, ModalFrame, ModalResponse, TextField, TextFieldKind};
use egui::{Align2, Context, CursorIcon, Id, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2};

use crate::component::coverage::CoverageBadge;
use crate::id::StudioUiId;
use crate::message::StudioUiMessage;
use crate::rule::LabelTint;
use crate::tokens::selector::StatusColor;

/// Paper: Menus + Dialogs, `Coverage panel` (`9GC-0` all, `9H2-0` Missing filter, `9HR-0` search). The status
/// filter pills with their counts and the search field, then the items by group: id, name, status badge, area and
/// "Go". The search matches a name, or an exact id. With nothing to list, the panel shows its empty look.
pub struct CoveragePanel<'a> {
    view: &'a CoveragePanelView,
    search: &'a mut String,
    field: Id,
}

impl<'a> CoveragePanel<'a> {
    /// Paper: the coverage card is 960 wide.
    const WIDTH: f32 = 960.0;
    // Paper: pills are 28 high with 12 each side, 8 apart; the selected pill is its colour at 25 %
    const PILL: f32 = 28.0;
    const PILL_PADDING: f32 = 12.0;
    const GAP: f32 = 8.0;
    const SELECTED_ALPHA: u8 = 0x40;
    // the search field takes its own row when less than this is left after the pills
    const SEARCH_MIN: f32 = 200.0;
    // Paper: rows and group heads are 30 high; columns: id 48, badge 130, area 60, go 40; 8 each side
    const ROW: f32 = 30.0;
    const ROW_PADDING: f32 = 8.0;
    const ID: f32 = 48.0;
    const BADGE: f32 = 130.0;
    const AREA: f32 = 60.0;
    const GO: f32 = 40.0;
    const EMPTY: f32 = 200.0;
    const LIST_TOP: f32 = 14.0;

    pub fn new(view: &'a CoveragePanelView, search: &'a mut String) -> Self {
        Self { view, search, field: Id::unique(StudioUiId::CoverageSearch) }
    }

    /// Another id for the search field, for a second panel on one screen (the gallery shows three).
    pub fn id(mut self, id: Id) -> Self {
        self.field = id;
        self
    }

    pub fn show(self, ctx: &Context) -> Option<CoveragePanelAction> {
        let frame = self.frame();
        let view = self.view;
        let (field, search) = (self.field, self.search);
        let mut body = None;
        let response = frame.show(ctx, |ui| body = Self::body(ui, view, field, search), |_| {});
        Self::action(response, body)
    }

    /// Shows the card where `ui` places it, for the gallery.
    pub fn show_card(self, ui: &mut Ui) -> Option<CoveragePanelAction> {
        let frame = self.frame();
        let view = self.view;
        let (field, search) = (self.field, self.search);
        let mut body = None;
        let response = frame.show_card(ui, |ui| body = Self::body(ui, view, field, search), |_| {});
        Self::action(response, body)
    }

    fn frame(&self) -> ModalFrame {
        ModalFrame::new(Id::unique(StudioUiId::CoveragePanel), Text::message(StudioUiMessage::Coverage))
            .subtitle(self.view.subtitle.clone())
            .width(Self::WIDTH)
    }

    fn body(ui: &mut Ui, view: &CoveragePanelView, field: Id, search: &mut String) -> Option<CoveragePanelAction> {
        let mut action = Self::filters(ui, view, field, search);
        // Paper: the list starts `LIST_TOP` under the filters, its rows packed with no gap
        ui.add_space(Self::LIST_TOP);
        ui.spacing_mut().item_spacing.y = 0.0;
        let query = search.trim().to_lowercase();
        let mut listed = false;
        for (group_index, group) in view.groups.iter().enumerate() {
            let rows: Vec<(usize, &CoverageRow)> = group
                .rows
                .iter()
                .enumerate()
                .filter(|(_, row)| view.filter.shows(row.status) && Self::matches(ui.ctx(), row, &query))
                .collect();
            if rows.is_empty() {
                continue;
            }
            listed = true;
            Self::head(ui, &group.title);
            for (row_index, row) in rows {
                if Self::row(ui, row) {
                    action = Some(CoveragePanelAction::GoTo { group: group_index, row: row_index });
                }
            }
        }
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let caption = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        if !listed {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::EMPTY), Sense::hover());
            let title = Text::message(StudioUiMessage::NoMatches);
            let message = Text::message(StudioUiMessage::NoMatchesHint);
            EmptyState::new(Icon::Search, title, message).show(&mut ui.new_child(UiBuilder::new().max_rect(rect)));
        } else if !query.is_empty() {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW), Sense::hover());
            let hint = Text::message(StudioUiMessage::SearchHint);
            let at = pos2(rect.left() + Self::ROW_PADDING, rect.center().y);
            Draw::text(ui.painter(), at, Align2::LEFT_CENTER, &hint, caption, faint, TextFit::Line);
        }
        action
    }

    // a row matches an empty search, a search inside its name, or its exact id
    fn matches(ctx: &Context, row: &CoverageRow, query: &str) -> bool {
        query.is_empty()
            || UiText::resolve(ctx, &row.label).to_lowercase().contains(query)
            || UiText::resolve(ctx, &row.id).to_lowercase() == query
    }

    // the filter pills with their counts, then the search field in the room left
    fn filters(
        ui: &mut Ui,
        view: &CoveragePanelView,
        search_id: Id,
        search: &mut String,
    ) -> Option<CoveragePanelAction> {
        let style = TextStyle::new(TypeSize::Caption, FontFace::Semibold);
        let width = ui.available_width();
        // the pills wrap onto more rows when the words are long; the search field follows them, on its own row
        // when less than `SEARCH_MIN` is left
        let mut pills = Vec::new();
        let (mut x, mut line) = (0.0_f32, 0_usize);
        for filter in CoverageFilter::ALL {
            let (hue, word) = match filter {
                CoverageFilter::All => (<color::Accent as ColorToken>::VALUE, StudioUiMessage::CoverageAll),
                CoverageFilter::Status(status) => (status.color(), Self::legend(status)),
            };
            let count = view.groups.iter().flat_map(|group| &group.rows).filter(|row| filter.shows(row.status)).count();
            let args = Args::new().message("label", word).uint("count", count as u64);
            let label = UiText::resolve(ui.ctx(), &Text::format(StudioUiMessage::FilterCount, args));
            let pill_width = (Draw::measure(ui.ctx(), &label, style).x + Self::PILL_PADDING * 2.0).min(width);
            if x > 0.0 && x + pill_width > width {
                (x, line) = (0.0, line + 1);
            }
            pills.push((filter, hue, label, x, line, pill_width));
            x += pill_width + Self::GAP;
        }
        if width - x < Self::SEARCH_MIN {
            (x, line) = (0.0, line + 1);
        }
        let height = Self::PILL * (line + 1) as f32 + Self::GAP * line as f32;
        let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let top = |line: usize| area.top() + (Self::PILL + Self::GAP) * line as f32;
        let mut action = None;
        for (filter, hue, label, left, pill_line, pill_width) in pills {
            let pill = Rect::from_min_size(pos2(area.left() + left, top(pill_line)), vec2(pill_width, Self::PILL));
            let response = ui.new_child(UiBuilder::new().max_rect(pill)).allocate_rect(pill, Sense::click());
            let response = response.on_hover_cursor(CursorIcon::PointingHand);
            let selected = view.filter == filter;
            let (fill, border, ink) = if selected {
                (hue.gamma_multiply_u8(Self::SELECTED_ALPHA), hue, <color::Text as ColorToken>::VALUE)
            } else if response.hovered() {
                (<color::BgHover as ColorToken>::VALUE, <color::LineStrong as ColorToken>::VALUE, LabelTint::of(hue))
            } else {
                (<color::BgField as ColorToken>::VALUE, <color::LineStrong as ColorToken>::VALUE, LabelTint::of(hue))
            };
            let painter = ui.painter();
            Draw::fill(painter, pill, Corner::Full, fill);
            painter.rect_stroke(pill, Corner::Full.radius(pill), Stroke::new(1.0, border), StrokeKind::Inside);
            let room = (pill_width - Self::PILL_PADDING * 2.0).max(0.0);
            let galley = Draw::galley(painter, &label, style, ink, TextFit::Truncate(room));
            painter.galley(Rect::from_center_size(pill.center(), galley.size()).min, galley, ink);
            if response.clicked() && !selected {
                action = Some(CoveragePanelAction::SetFilter(filter));
            }
        }
        let field = Rect::from_min_max(pos2(area.left() + x, top(line)), pos2(area.right(), top(line) + Self::PILL));
        let placeholder = Text::message(StudioUiMessage::SearchItems);
        let mut child = ui.new_child(UiBuilder::new().max_rect(field));
        TextField::new(search_id, search, TextFieldKind::Search, placeholder).show(&mut child, field.width());
        action
    }

    fn head(ui: &mut Ui, title: &Text) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW), Sense::hover());
        let style = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
        let faint = <color::TextFaint as ColorToken>::VALUE;
        Draw::text(
            ui.painter(),
            rect.left_center(),
            Align2::LEFT_CENTER,
            title,
            style,
            faint,
            TextFit::Truncate(rect.width()),
        );
    }

    // one item row; true when its "Go" was clicked
    fn row(ui: &mut Ui, row: &CoverageRow) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW), Sense::hover());
        let painter = ui.painter().clone();
        if response.hovered() {
            Draw::fill(&painter, rect, Corner::Control, <color::BgHover as ColorToken>::VALUE);
        }
        let inner = rect.shrink2(vec2(Self::ROW_PADDING, 0.0));
        let y = rect.center().y;
        let caption = TextStyle::new(TypeSize::Caption, FontFace::Regular);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let faint = <color::TextFaint as ColorToken>::VALUE;
        Draw::text(
            &painter,
            pos2(inner.left(), y),
            Align2::LEFT_CENTER,
            &row.id,
            caption,
            dim,
            TextFit::Truncate(Self::ID),
        );
        let go = Rect::from_min_size(pos2(inner.right() - Self::GO, rect.top()), vec2(Self::GO, Self::ROW));
        let area = Rect::from_min_size(pos2(go.left() - Self::AREA, rect.top()), vec2(Self::AREA, Self::ROW));
        let badge_column =
            Rect::from_min_size(pos2(area.left() - Self::BADGE, rect.top()), vec2(Self::BADGE, Self::ROW));
        let label_left = inner.left() + Self::ID;
        let note = TextStyle::new(TypeSize::Note, FontFace::Regular);
        let ink = <color::Text as ColorToken>::VALUE;
        let room = (badge_column.left() - Self::GAP - label_left).max(0.0);
        Draw::text(&painter, pos2(label_left, y), Align2::LEFT_CENTER, &row.label, note, ink, TextFit::Truncate(room));
        let badge = CoverageBadge::new(row.status, Text::message(Self::legend(row.status)));
        let size = badge.size(ui.ctx());
        let badge_rect = Rect::from_min_size(
            pos2(badge_column.left(), y - size.y / 2.0),
            vec2(size.x.min(Self::BADGE - Self::GAP), size.y),
        );
        badge.paint(&painter, badge_rect);
        Draw::text(
            &painter,
            pos2(area.left(), y),
            Align2::LEFT_CENTER,
            &row.area,
            caption,
            faint,
            TextFit::Truncate(Self::AREA),
        );
        if !row.go {
            return false;
        }
        let response = ui.new_child(UiBuilder::new().max_rect(go)).allocate_rect(go, Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        if response.hovered() {
            Draw::fill(&painter, go.shrink(2.0), Corner::Small, <color::BgActive as ColorToken>::VALUE);
        }
        let help = TextStyle::new(TypeSize::Help, FontFace::Regular);
        Draw::text(
            &painter,
            go.center(),
            Align2::CENTER_CENTER,
            &Text::message(StudioUiMessage::GoTo),
            help,
            ink,
            TextFit::Line,
        );
        response.clicked()
    }

    fn legend(status: StatusColor) -> StudioUiMessage {
        match status {
            StatusColor::Works => StudioUiMessage::LegendWorks,
            StatusColor::Engine => StudioUiMessage::LegendEngine,
            StatusColor::Planned => StudioUiMessage::LegendPlanned,
            StatusColor::Mockup => StudioUiMessage::LegendMockup,
            StatusColor::Missing => StudioUiMessage::LegendMissing,
        }
    }

    fn action(response: ModalResponse, action: Option<CoveragePanelAction>) -> Option<CoveragePanelAction> {
        if response.closed { Some(CoveragePanelAction::Close) } else { action }
    }
}

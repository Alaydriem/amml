use egui::{Frame, Rangef, Rect, ScrollArea, Sense, Ui, vec2};

use crate::draw::{Corner, Draw, TextStyle, TypeSize};
use crate::font::FontFace;
use crate::gallery::{GalleryState, SheetGrid};
use crate::state::InteractionState;
use crate::tokens::{ColorToken, SpacingToken, color, spacing};
use crate::widget::{ScrollBar, Splitter};

/// Paper: Editor, `Editor / Splitter + scroll` (`BWO-0`); Mod manager, the mod list's scroll bar
/// (`FGA-0`). The splitter between a viewer and a transport in its three looks, and a scroll area
/// with and without the track. The lane rows are plain stand-in rows.
pub struct SplitterAndScrollSheet;

impl SplitterAndScrollSheet {
    /// Sheet chrome: a preview's width, the viewer and transport heights in it, the scroll area's
    /// height and its row count.
    const PREVIEW_WIDTH: f32 = 320.0;
    const PANE: f32 = 48.0;
    const SCROLL_HEIGHT: f32 = 240.0;
    const ROWS: usize = 30;

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Splitter + scroll", |ui| {
            SheetGrid::section(ui, "SPLITTER · VIEWER ABOVE, TRANSPORT BELOW", |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP + SheetGrid::COLUMN_GAP, 0.0);
                    let looks = [
                        ("REST", InteractionState::Rest),
                        ("HOVER", InteractionState::Hover),
                        ("DRAGGING", InteractionState::Pressed),
                    ];
                    for (head, state) in looks {
                        SheetGrid::column(ui, head, Self::PREVIEW_WIDTH, |ui| Self::preview(ui, state));
                    }
                });
            });
            SheetGrid::section(ui, "SCROLL BAR", |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = vec2(SheetGrid::COLUMN_GAP + SheetGrid::COLUMN_GAP, 0.0);
                    for (head, track) in [("MOD LIST · WITH TRACK", true), ("LANE AREA · NO TRACK", false)] {
                        SheetGrid::column(ui, head, Self::PREVIEW_WIDTH, |ui| Self::scroll(ui, head, track));
                    }
                });
            });
        });
    }

    // a viewer, the splitter in `state`, and a transport, in a small frame
    fn preview(ui: &mut Ui, state: InteractionState) {
        Frame::new()
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Small.radius(Rect::NOTHING))
            .show(ui, |ui| {
                let splitter = <spacing::Splitter as SpacingToken>::VALUE;
                let height = 2.0 * Self::PANE + splitter;
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
                let painter = ui.painter_at(rect);
                let viewer = Rect::from_min_size(rect.min, vec2(rect.width(), Self::PANE));
                Draw::fill(&painter, viewer, Corner::None, <color::BgApp as ColorToken>::VALUE);
                let bar = Rect::from_min_size(viewer.left_bottom(), vec2(rect.width(), splitter));
                Splitter::paint(&painter, bar, state);
                let transport = Rect::from_min_max(bar.left_bottom(), rect.max);
                Draw::fill(&painter, transport, Corner::None, <color::BgRaised as ColorToken>::VALUE);
                let line = <color::Line as ColorToken>::VALUE;
                Draw::divider(&painter, Rangef::new(transport.left(), transport.right()), transport.top(), line);
            });
    }

    // a panel of stand-in rows that scrolls, with or without the track
    fn scroll(ui: &mut Ui, id: &str, track: bool) {
        Frame::new()
            .fill(<color::BgPanel as ColorToken>::VALUE)
            .stroke(Draw::border(<color::Line as ColorToken>::VALUE))
            .corner_radius(Corner::Panel.radius(Rect::NOTHING))
            .inner_margin(Draw::margin(<spacing::S as SpacingToken>::VALUE, <spacing::S as SpacingToken>::VALUE))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let area = ScrollArea::vertical()
                    .id_salt(id)
                    .min_scrolled_height(Self::SCROLL_HEIGHT)
                    .max_height(Self::SCROLL_HEIGHT)
                    .auto_shrink(false);
                ScrollBar::show(ui, area, track, |ui| {
                    let style = TextStyle::new(TypeSize::Note, FontFace::Regular);
                    for index in 1..=Self::ROWS {
                        let row = format!("Row {index}");
                        SheetGrid::label(ui, &row, style, <color::Text as ColorToken>::VALUE);
                    }
                });
            });
    }
}

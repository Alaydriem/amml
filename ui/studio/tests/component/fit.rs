use amml_studio_ui::component::bar::{
    SceneSwitcherChip, StatusBar, StatusBarLayout, StatusBarView, StudioMode, TopBar, TopBarView, Transport,
    TransportLayout, TransportView,
};
use amml_studio_ui::component::dialog::{
    ConfirmDialog, ConfirmDialogView, CoverageFilter, CoverageGroup, CoveragePanel, CoveragePanelView, CoverageRow,
    RenameDialog,
};
use amml_studio_ui::component::inspector::{
    InspectorRail, InspectorTab, InspectorTabs, UndoHistoryPopover, UndoHistoryView,
};
use amml_studio_ui::component::timeline::{TimelineView, TrackHeader};
use amml_studio_ui::tokens::selector::{ModeColor, StatusColor};
use amml_studio_ui::tokens::spacing as studio_spacing;
use amml_ui::tokens::SpacingToken;
use egui::{Rect, Shape, Ui, UiBuilder, pos2, vec2};

use super::bar::arrival;
use super::timeline::{model, track};
use crate::glyph::words;
use crate::support::UiProbe;

// Review Focus 1 and 2: in the small window and in en_XA, every part keeps its text and its rect to itself.

// the texts that start inside `part` and end past its right edge
fn outside(shapes: &[Shape], part: Rect) -> Vec<(String, Rect)> {
    shapes
        .iter()
        .filter_map(|shape| match shape {
            Shape::Text(text) => Some((text.galley.text().to_owned(), text.galley.rect.translate(text.pos.to_vec2()))),
            _ => None,
        })
        .filter(|(_, rect)| part.contains(rect.left_top()))
        .filter(|(_, rect)| rect.right() > part.right() + 0.5)
        .collect()
}

// shows `part` in an en_XA probe at the small window and returns the rect it took and what it painted
fn pseudo(width: f32, mut part: impl FnMut(&mut Ui)) -> (Rect, Vec<Shape>) {
    let mut probe = UiProbe::pseudo(1280.0, 720.0);
    let mut show = |ui: &mut Ui| {
        let rect = Rect::from_min_size(ui.cursor().min, vec2(width, 720.0));
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            part(ui);
            ui.min_rect()
        })
        .inner
    };
    probe.frame(Vec::new(), &mut show);
    probe.frame(Vec::new(), &mut show)
}

fn top_bar() -> TopBarView {
    TopBarView {
        title: words("Nether trip"),
        mode: StudioMode::Edit,
        can_undo: true,
        can_redo: false,
        history_open: false,
        scene: arrival(),
        export_blocked: None,
    }
}

fn transport() -> TransportView {
    TransportView {
        playing: false,
        timecode: words("00:00:32.95"),
        duration: Some(words("02:14:08.40")),
        speed: words("1.00×"),
        looping: false,
        snap: true,
        guides: false,
        zoom: 0.5,
    }
}

fn status_bar() -> StatusBarView {
    StatusBarView {
        mode: ModeColor::Edit,
        mode_text: words("Edit · free camera"),
        detail: words("Rewind cache 1609 checkpoints"),
        shortcuts: words("UI 100%"),
        tinted: false,
    }
}

#[test]
fn pseudo_text_stays_in_its_part() {
    // (name, width, the part)
    type Check = (&'static str, f32, Box<dyn FnMut(&mut Ui)>);
    let checks: Vec<Check> = vec![
        (
            "top bar",
            1280.0,
            Box::new(|ui| {
                TopBar::new(&top_bar()).show(ui);
            }),
        ),
        (
            "transport",
            1280.0,
            Box::new(|ui| {
                Transport::new(&transport()).layout(TransportLayout::Compact).show(ui);
            }),
        ),
        (
            "status bar",
            1280.0,
            Box::new(|ui| {
                StatusBar::new(&status_bar()).layout(StatusBarLayout::Compact).show(ui);
            }),
        ),
        (
            "scene switcher chip",
            400.0,
            Box::new(|ui| {
                SceneSwitcherChip::new(&arrival()).show(ui);
            }),
        ),
        (
            "track header",
            TrackHeader::WIDTH,
            Box::new(|ui| {
                let view = track("Time of day", 12, true).header;
                let rect = Rect::from_min_size(ui.cursor().min, vec2(TrackHeader::WIDTH, 64.0));
                ui.allocate_rect(rect, egui::Sense::hover());
                TrackHeader::new(&view).show_at(ui, rect);
            }),
        ),
        (
            "inspector tabs",
            430.0,
            Box::new(|ui| {
                InspectorTabs::new(InspectorTab::Players).show(ui);
            }),
        ),
        (
            "undo history",
            UndoHistoryPopover::WIDTH,
            Box::new(|ui| {
                UndoHistoryPopover::new(&UndoHistoryView { steps: Vec::new() }).show(ui);
            }),
        ),
        (
            "confirm dialog",
            480.0,
            Box::new(|ui| {
                let view = ConfirmDialogView {
                    title: words("Remove the Camera track?"),
                    subtitle: words("Undo can bring it back"),
                    body: words("It has 6 keys."),
                    confirm: words("Remove track"),
                };
                ConfirmDialog::new(&view).show_card(ui);
            }),
        ),
        (
            "rename dialog",
            460.0,
            Box::new(|ui| {
                let title = words("Rename scene");
                let mut value = "Portal run".to_owned();
                RenameDialog::new(&title, &mut value).show_card(ui);
            }),
        ),
        (
            "coverage panel",
            960.0,
            Box::new(|ui| {
                let view = CoveragePanelView {
                    subtitle: words("Coverage"),
                    filter: CoverageFilter::All,
                    groups: vec![CoverageGroup {
                        title: words("A. Library and open replay"),
                        rows: vec![CoverageRow {
                            id: words("A14"),
                            label: words("Rename a replay"),
                            status: StatusColor::Missing,
                            area: words("Players tab"),
                            go: true,
                        }],
                    }],
                };
                let mut search = String::new();
                CoveragePanel::new(&view, &mut search).show_card(ui);
            }),
        ),
    ];
    for (name, width, part) in checks {
        let (rect, shapes) = pseudo(width, part);
        let outside = outside(&shapes, rect);
        assert!(outside.is_empty(), "{name} texts past its edge {rect:?}: {outside:?}");
    }
}

#[test]
fn bars_leave_the_cursor_under_them() {
    let (top, transport) = (top_bar(), self::transport());
    let mut probe = UiProbe::sized(1280.0, 720.0);
    let mut show = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        let top_bar = ui.cursor().min.y;
        TopBar::new(&top).show(ui);
        let under_top_bar = ui.cursor().min.y;
        let transport_top = ui.cursor().min.y;
        Transport::new(&transport).show(ui);
        (under_top_bar - top_bar, ui.cursor().min.y - transport_top)
    };
    probe.frame(Vec::new(), &mut show);
    let ((top_height, transport_height), _) = probe.frame(Vec::new(), &mut show);
    assert_eq!(top_height, <studio_spacing::TopBar as SpacingToken>::VALUE, "the part under the top bar");
    assert_eq!(transport_height, <studio_spacing::Transport as SpacingToken>::VALUE, "the part under the transport");
}

#[test]
fn small_window_parts_stay_in_their_rects() {
    // 1280 is the first width with the Full bars; 1279 has the Compact ones
    for width in [1280.0, 1279.0] {
        small_window(width);
    }
}

// the studio's parts stacked in a `width` × 720 window
fn small_window(width: f32) {
    let screen = Rect::from_min_size(pos2(0.0, 0.0), vec2(width, 720.0));
    let (top, transport, status) = (top_bar(), self::transport(), status_bar());
    let timeline = {
        let mut model = model(6);
        model.tracks.push(track("Weather", 2, false));
        model
    };
    let mut probe = UiProbe::sized(width, 720.0);
    let mut show = |ui: &mut Ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        let mut parts = Vec::new();
        // each part gets its band of the screen; what it records is the rect it took
        let part = |ui: &mut Ui, name: &'static str, height: f32, body: &mut dyn FnMut(&mut Ui)| {
            let rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), height));
            let used = ui
                .scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                    body(ui);
                    ui.min_rect()
                })
                .inner;
            ui.advance_cursor_after_rect(rect);
            (name, used)
        };
        parts.push(part(ui, "top bar", 46.0, &mut |ui| {
            TopBar::new(&top).show(ui);
        }));
        let middle = Rect::from_min_size(ui.cursor().min, vec2(width, 720.0 - 46.0 - 260.0 - 50.0 - 26.0));
        let rail_width = 44.0;
        let viewer = Rect::from_min_max(middle.min, pos2(middle.right() - rail_width, middle.bottom()));
        let rail = Rect::from_min_max(pos2(viewer.right(), middle.top()), middle.max);
        let rail_used = ui
            .scope_builder(UiBuilder::new().max_rect(rail), |ui| {
                InspectorRail::new(InspectorTab::Key).show(ui);
                ui.min_rect()
            })
            .inner;
        ui.advance_cursor_after_rect(middle);
        parts.push(("viewer", viewer));
        parts.push(("inspector rail", rail_used));
        parts.push(part(ui, "timeline", 260.0, &mut |ui| {
            TimelineView::new(&timeline).show(ui);
        }));
        parts.push(part(ui, "transport", 50.0, &mut |ui| {
            Transport::new(&transport).layout(TransportLayout::for_width(width)).show(ui);
        }));
        parts.push(part(ui, "status bar", 26.0, &mut |ui| {
            StatusBar::new(&status).layout(StatusBarLayout::for_width(width)).show(ui);
        }));
        parts
    };
    probe.frame(Vec::new(), &mut show);
    let (parts, shapes) = probe.frame(Vec::new(), &mut show);
    for (name, rect) in &parts {
        assert!(screen.contains_rect(*rect), "{name} {rect:?} leaves the screen");
        let outside = outside(&shapes, *rect);
        assert!(outside.is_empty(), "{name} texts past its edge: {outside:?}");
    }
    for (index, (name, rect)) in parts.iter().enumerate() {
        for (other, other_rect) in &parts[index + 1..] {
            let overlap = rect.intersect(*other_rect);
            assert!(!overlap.is_positive(), "{name} {rect:?} overlaps {other} {other_rect:?}");
        }
    }
}

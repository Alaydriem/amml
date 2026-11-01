use amml_ui::gallery::{Sample, SheetGrid};
use amml_ui::icon::Icon;
use egui::Ui;

use crate::component::bar::{
    SceneSwitcherFooter, SceneSwitcherList, SceneSwitcherListView, SceneSwitcherOriginal, SceneSwitcherRow,
};
use crate::gallery::GalleryState;
use crate::tokens::selector::SceneColor;

/// Paper: Editor components, `Editor / Scene switcher list` (`GFV-0`). The open list: the Original row, the active
/// scene, scenes in three statuses and the actions.
pub struct SceneSwitcherListPage;

impl SceneSwitcherListPage {
    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        SheetGrid::page(ui, "Scene switcher list", |ui| {
            let scene = |name: &str, range: &str, cameras: &str, status: &str, color, active| SceneSwitcherRow {
                name: Sample::text(name),
                range: Sample::text(range),
                cameras: Sample::text(cameras),
                status: Sample::text(status),
                color,
                active,
            };
            let action = |icon, label: &str, shortcut: Option<&str>| SceneSwitcherFooter {
                icon,
                label: Sample::text(label),
                shortcut: shortcut.map(Sample::text),
            };
            let view = SceneSwitcherListView {
                header: Sample::text("SCENES IN THIS REPLAY"),
                hint: Sample::text("right-click a scene for more"),
                original: SceneSwitcherOriginal {
                    name: Sample::text("Original (as recorded)"),
                    detail: Sample::text("no overrides, FOV 72"),
                    active: false,
                },
                scenes: vec![
                    scene("Arrival", "00:28.80 – 00:37.40", "6 cam", "Ready", SceneColor::Lime, true),
                    scene("Portal run", "00:37.80 – 00:52.00", "4 cam", "Rendered", SceneColor::Pink, false),
                    scene("Fortress", "00:12.40 – 00:28.40", "3 cam", "Draft", SceneColor::Orchid, false),
                    scene("Multi-angle B", "00:31.20 – 00:36.60", "2 cam", "Draft", SceneColor::Periwinkle, false),
                    scene("Blaze farm", "00:55.00 – 01:04.60", "0 cam", "Ready", SceneColor::Peach, false),
                ],
                footer: vec![
                    action(Icon::Add, "New scene at playhead", Some("Ctrl+Shift+N")),
                    action(Icon::Checklist, "New scene from selected keys", None),
                    action(Icon::Flag, "New scene between bookmarks", None),
                    action(Icon::Copy, "Duplicate active scene", Some("Ctrl+D")),
                ],
            };
            SheetGrid::section(ui, "OPEN", |ui| {
                SceneSwitcherList::new(&view).show(ui);
            });
        });
    }
}

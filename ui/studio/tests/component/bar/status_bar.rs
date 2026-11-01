use amml_studio_ui::component::bar::{StatusBar, StatusBarLayout, StatusBarView};
use amml_studio_ui::tokens::selector::ModeColor;
use amml_ui::tokens::{SpacingToken, spacing};

use crate::glyph::words;
use crate::support::UiProbe;

fn view() -> StatusBarView {
    StatusBarView {
        mode: ModeColor::Edit,
        mode_text: words("Edit · free camera"),
        detail: words("Rewind cache 1609 checkpoints · no seek yet"),
        shortcuts: words("UI 100%"),
        tinted: false,
    }
}

#[test]
fn status_bar_is_compact_below_the_small_breakpoint() {
    assert_eq!(StatusBarLayout::for_width(1279.0), StatusBarLayout::Compact);
    assert_eq!(StatusBarLayout::for_width(1280.0), StatusBarLayout::Full);
}

#[test]
fn status_bar_fills_its_token_height() {
    let mut probe = UiProbe::new(1920.0);
    let view = view();
    let (height, _) = probe.frame(Vec::new(), |ui| {
        StatusBar::new(&view).show(ui);
        ui.min_rect().height()
    });
    assert_eq!(height, <spacing::StatusBar as SpacingToken>::VALUE);
}

#[test]
fn compact_status_bar_drops_the_detail() {
    let view = view();
    let full = UiProbe::texts(&UiProbe::new(1920.0).shapes(|ui| StatusBar::new(&view).show(ui)));
    let compact = UiProbe::texts(
        &UiProbe::new(1280.0).shapes(|ui| StatusBar::new(&view).layout(StatusBarLayout::Compact).show(ui)),
    );
    let detail = "Rewind cache 1609 checkpoints · no seek yet";
    assert!(full.iter().any(|text| text == detail), "{full:?}");
    assert!(!compact.iter().any(|text| text == detail), "{compact:?}");
    assert!(compact.iter().any(|text| text == "MISSING"), "the legend stays: {compact:?}");
}

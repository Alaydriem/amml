use amml_studio_ui::component::bar::{Transport, TransportAction, TransportControl, TransportLayout, TransportView};
use amml_studio_ui::tokens::spacing;
use amml_ui::icon::{Icon, IconShape};
use amml_ui::tokens::SpacingToken;

use crate::glyph::words;
use crate::support::UiProbe;

fn view(playing: bool) -> TransportView {
    TransportView {
        playing,
        timecode: words("00:00:32.95"),
        duration: None,
        speed: words("1x"),
        looping: false,
        snap: true,
        guides: false,
        zoom: 0.65,
    }
}

fn glyph(icon: Icon) -> String {
    match icon.shape() {
        IconShape::Font(code) => code.to_string(),
        IconShape::Drawn(_) => panic!("{icon:?} is drawn"),
    }
}

#[test]
fn transport_is_compact_below_the_small_breakpoint() {
    assert_eq!(TransportLayout::for_width(1279.0), TransportLayout::Compact);
    assert_eq!(TransportLayout::for_width(1280.0), TransportLayout::Full);
}

#[test]
fn transport_fills_its_token_height() {
    for layout in [TransportLayout::Full, TransportLayout::Compact] {
        let mut probe = UiProbe::new(1490.0);
        let view = view(false);
        let (height, _) = probe.frame(Vec::new(), |ui| {
            Transport::new(&view).layout(layout).show(ui);
            ui.min_rect().height()
        });
        assert_eq!(height, <spacing::Transport as SpacingToken>::VALUE, "{layout:?}");
    }
}

#[test]
fn play_click_sends_toggle_play() {
    for (playing, icon) in [(false, Icon::Play), (true, Icon::Pause)] {
        let mut probe = UiProbe::new(1490.0);
        let view = view(playing);
        let shapes = probe.shapes(|ui| Transport::new(&view).show(ui));
        let at = UiProbe::text_rect(&shapes, &glyph(icon)).unwrap_or_else(|| panic!("no {icon:?}")).center();
        assert_eq!(probe.click(at, |ui| Transport::new(&view).show(ui)), vec![TransportAction::TogglePlay]);
    }
}

#[test]
fn add_camera_key_is_disabled_while_playing() {
    for (playing, expected) in [(false, vec![TransportAction::AddCameraKey]), (true, Vec::new())] {
        let mut probe = UiProbe::new(1490.0);
        let view = view(playing);
        let shapes = probe.shapes(|ui| Transport::new(&view).show(ui));
        let at = UiProbe::text_rect(&shapes, "Add camera key").unwrap_or_else(|| panic!("no add key")).center();
        assert_eq!(probe.click(at, |ui| Transport::new(&view).show(ui)), expected, "playing {playing}");
    }
}

#[test]
fn transport_has_no_timelapse_control() {
    for layout in [TransportLayout::Full, TransportLayout::Compact] {
        let mut probe = UiProbe::new(1490.0);
        let view = view(false);
        let shapes = probe.shapes(|ui| Transport::new(&view).layout(layout).show(ui));
        let texts = UiProbe::texts(&shapes);
        assert!(texts.iter().all(|text| !text.to_lowercase().contains("timelapse")), "{texts:?}");
    }
}

#[test]
fn compact_transport_drops_the_add_key_and_fit_words() {
    let mut probe = UiProbe::new(1236.0);
    let view = view(false);
    let shapes = probe.shapes(|ui| Transport::new(&view).layout(TransportLayout::Compact).show(ui));
    let texts = UiProbe::texts(&shapes);
    assert!(!texts.iter().any(|text| text == "Add camera key" || text == "Fit"), "{texts:?}");
    assert!(texts.contains(&glyph(Icon::Fit)), "the compact Fit is an icon: {texts:?}");
}

#[test]
fn the_duration_shows_after_the_timecode_when_given() {
    let mut probe = UiProbe::new(1920.0);
    let mut view = view(false);
    view.duration = Some(words("02:14:08.40"));
    let shapes = probe.shapes(|ui| Transport::new(&view).show(ui));
    let time = UiProbe::text_rect(&shapes, "00:00:32.95").unwrap_or_else(|| panic!("no timecode"));
    let duration = UiProbe::text_rect(&shapes, "/ 02:14:08.40").unwrap_or_else(|| panic!("no duration"));
    assert!(duration.left() > time.right());
}

#[test]
fn narrow_compact_transport_keeps_its_groups_apart() {
    for width in [760.0, 900.0] {
        let mut probe = UiProbe::new(width);
        let view = view(false);
        let show = |ui: &mut egui::Ui| Transport::new(&view).layout(TransportLayout::Compact).show(ui);
        let shapes = probe.shapes(show);
        let guides = UiProbe::text_rect(&shapes, &glyph(Icon::Grid)).unwrap_or_else(|| panic!("no guides at {width}"));
        for icon in [Icon::ZoomOut, Icon::ZoomIn] {
            if let Some(rect) = UiProbe::text_rect(&shapes, &glyph(icon)) {
                assert!(rect.left() > guides.right(), "{icon:?} {rect:?} over the guides {guides:?} at {width}");
            }
        }
        assert_eq!(probe.click(guides.center(), show), vec![TransportAction::ToggleGuides], "at {width}");
    }
}

fn placed(width: f32, layout: TransportLayout) -> (egui::Rect, Vec<(TransportControl, egui::Rect)>) {
    let mut probe = UiProbe::new(width);
    let view = view(false);
    let show = |ui: &mut egui::Ui| {
        let (_, places) = Transport::new(&view).layout(layout).show_placed(ui);
        (ui.min_rect(), places)
    };
    probe.frame(Vec::new(), show);
    probe.frame(Vec::new(), show).0
}

fn action_of(control: TransportControl) -> Option<TransportAction> {
    Some(match control {
        TransportControl::ToSceneStart => TransportAction::ToSceneStart,
        TransportControl::PreviousKey => TransportAction::PreviousKey,
        TransportControl::BackFive => TransportAction::BackFive,
        TransportControl::Play => TransportAction::TogglePlay,
        TransportControl::ForwardFive => TransportAction::ForwardFive,
        TransportControl::NextKey => TransportAction::NextKey,
        TransportControl::ToSceneEnd => TransportAction::ToSceneEnd,
        TransportControl::Speed => TransportAction::CycleSpeed,
        TransportControl::Loop => TransportAction::ToggleLoop,
        TransportControl::Snap => TransportAction::ToggleSnap,
        TransportControl::Guides => TransportAction::ToggleGuides,
        TransportControl::ZoomOut => TransportAction::ZoomOut,
        TransportControl::ZoomIn => TransportAction::ZoomIn,
        TransportControl::AddCameraKey => TransportAction::AddCameraKey,
        TransportControl::Fit => TransportAction::Fit,
        TransportControl::Timecode | TransportControl::ZoomSlider => return None,
    })
}

const ALL_CONTROLS: [TransportControl; 17] = [
    TransportControl::ToSceneStart,
    TransportControl::PreviousKey,
    TransportControl::BackFive,
    TransportControl::Play,
    TransportControl::ForwardFive,
    TransportControl::NextKey,
    TransportControl::ToSceneEnd,
    TransportControl::Timecode,
    TransportControl::Speed,
    TransportControl::Loop,
    TransportControl::Snap,
    TransportControl::Guides,
    TransportControl::ZoomOut,
    TransportControl::ZoomSlider,
    TransportControl::ZoomIn,
    TransportControl::AddCameraKey,
    TransportControl::Fit,
];

#[test]
fn full_transport_places_all_17_controls_inside_the_bar() {
    let (bar, places) = placed(1920.0, TransportLayout::Full);
    assert_eq!(places.len(), 17, "{places:?}");
    for control in ALL_CONTROLS {
        let rect = places.iter().find(|(c, _)| *c == control).unwrap_or_else(|| panic!("no {control:?}")).1;
        assert!(bar.contains_rect(rect), "{control:?} {rect:?} outside {bar:?}");
    }
}

#[test]
fn a_click_at_the_centre_of_each_placed_control_gives_its_action() {
    let (_, places) = placed(1920.0, TransportLayout::Full);
    for (control, rect) in places {
        let Some(expected) = action_of(control) else { continue };
        let mut probe = UiProbe::new(1920.0);
        let view = view(false);
        let show = |ui: &mut egui::Ui| Transport::new(&view).show(ui);
        probe.shapes(show);
        assert_eq!(probe.click(rect.center(), show), vec![expected], "{control:?}");
    }
}

#[test]
fn compact_transport_with_no_room_omits_the_zoom_controls() {
    let (_, places) = placed(560.0, TransportLayout::Compact);
    for control in [TransportControl::ZoomOut, TransportControl::ZoomSlider, TransportControl::ZoomIn] {
        assert!(places.iter().all(|(c, _)| *c != control), "{control:?} listed: {places:?}");
    }
    for control in [TransportControl::Play, TransportControl::Speed, TransportControl::Fit] {
        assert!(places.iter().any(|(c, _)| *c == control), "{control:?} missing");
    }
}

#[test]
fn show_and_show_placed_give_the_same_actions() {
    let mut probe = UiProbe::new(1920.0);
    let view = view(false);
    let shapes = probe.shapes(|ui| Transport::new(&view).show(ui));
    let at = UiProbe::text_rect(&shapes, &glyph(Icon::Play)).unwrap().center();
    let placed = probe.click(at, |ui| Transport::new(&view).show_placed(ui).0);
    assert_eq!(placed, vec![TransportAction::TogglePlay]);
}

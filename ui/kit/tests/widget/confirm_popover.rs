use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::widget::{ConfirmPopover, ConfirmPopoverChoice};
use egui::{Key, Rect, pos2, vec2};
use egui_kittest::Harness;

use crate::support::Sample;

// shows the discard popover beside a rail tile each pass and keeps the last choice it reported
struct PopoverProbe;

impl PopoverProbe {
    fn harness() -> Harness<'static, Option<ConfirmPopoverChoice>> {
        let mut harness = Harness::builder()
            .with_size(vec2(1000.0, 700.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                |ui, last: &mut Option<ConfirmPopoverChoice>| {
                    let anchor = Rect::from_min_size(pos2(900.0, 300.0), vec2(72.0, 72.0));
                    let popover = ConfirmPopover::new(Sample::text("Discard this recording?"), Sample::text("Discard"))
                        .body(Sample::text("12:41 of capture will be deleted. You cannot undo this."))
                        .cancel(Sample::text("Keep recording"));
                    if let Some(choice) = popover.show(ui, anchor) {
                        *last = Some(choice);
                    }
                },
                None,
            );
        harness.run();
        harness
    }
}

#[test]
fn confirm_popover_enter_confirms() {
    let mut harness = PopoverProbe::harness();
    assert_eq!(*harness.state(), None);
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(*harness.state(), Some(ConfirmPopoverChoice::Confirm));
}

// what the focus probe saw: the popover's last choice, the field's words, and whether Escape reached the page
#[derive(Default)]
struct FocusState {
    choice: Option<ConfirmPopoverChoice>,
    field: String,
    escape_after: bool,
}

// the popover over a page with a focused text field
fn focus_harness() -> Harness<'static, FocusState> {
    let mut harness = Harness::builder()
        .with_size(vec2(1000.0, 700.0))
        .allow_missing_glyphs()
        // the bare test field has no label
        .with_accessibility_check(false)
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui_state(
            |ui, state: &mut FocusState| {
                ui.text_edit_singleline(&mut state.field).request_focus();
                let anchor = Rect::from_min_size(pos2(900.0, 300.0), vec2(72.0, 72.0));
                let popover = ConfirmPopover::new(Sample::text("Discard this recording?"), Sample::text("Discard"));
                if let Some(choice) = popover.show(ui, anchor) {
                    state.choice = Some(choice);
                }
                state.escape_after |= ui.input(|input| input.key_pressed(Key::Escape));
            },
            FocusState::default(),
        );
    harness.run();
    harness
}

#[test]
fn confirm_popover_enter_in_a_focused_field_does_not_confirm() {
    let mut harness = focus_harness();
    harness.key_press(Key::Enter);
    harness.run();
    assert_ne!(harness.state().choice, Some(ConfirmPopoverChoice::Confirm));
}

#[test]
fn confirm_popover_takes_the_escape_it_answers() {
    let mut harness = focus_harness();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().choice, Some(ConfirmPopoverChoice::Cancel));
    assert!(!harness.state().escape_after, "the Escape also reached the page under the popover");
}

#[test]
fn confirm_popover_ignores_an_enter_in_the_frame_it_opens() {
    let mut harness = Harness::builder()
        .with_size(vec2(1000.0, 700.0))
        .allow_missing_glyphs()
        .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
        .build_ui_state(
            |ui, (open, choice): &mut (bool, Option<ConfirmPopoverChoice>)| {
                if *open {
                    let anchor = Rect::from_min_size(pos2(900.0, 300.0), vec2(72.0, 72.0));
                    let popover = ConfirmPopover::new(Sample::text("Discard this recording?"), Sample::text("Discard"));
                    *choice = popover.show(ui, anchor).or(*choice);
                }
            },
            (false, None),
        );
    harness.run();
    harness.state_mut().0 = true;
    harness.key_press(Key::Enter);
    harness.step();
    assert_eq!(harness.state().1, None, "the Enter that opened the popover answered it");
}

#[test]
fn confirm_popover_escape_cancels() {
    let mut harness = PopoverProbe::harness();
    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(*harness.state(), Some(ConfirmPopoverChoice::Cancel));
}

// a caller that routes clicks needs the card's rect: `rect` is where `show` puts it, also flipped at the left edge
#[test]
fn confirm_popover_rect_is_where_show_puts_it() {
    for anchor in [pos2(900.0, 300.0), pos2(0.0, 650.0)] {
        let anchor = Rect::from_min_size(anchor, vec2(72.0, 72.0));
        let id = egui::Id::unique("confirm-popover-rect-test");
        let mut harness = Harness::builder()
            .with_size(vec2(1000.0, 700.0))
            .allow_missing_glyphs()
            .with_context_setup(|ctx| Theme::install(ctx, FontSource::BuiltIn))
            .build_ui_state(
                |ui, rect: &mut Rect| {
                    let popover = ConfirmPopover::new(Sample::text("Discard this recording?"), Sample::text("Discard"))
                        .body(Sample::text("The capture so far will be deleted."))
                        .id(id);
                    *rect = popover.rect(ui.ctx(), anchor);
                    popover.show(ui, anchor);
                },
                Rect::NOTHING,
            );
        harness.run();

        let shown = harness.ctx.memory(|memory| memory.area_rect(id)).unwrap_or(Rect::NOTHING);
        assert_eq!(*harness.state(), shown, "anchor {anchor:?}");
    }
}

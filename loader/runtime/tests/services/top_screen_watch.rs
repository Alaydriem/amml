use amml_runtime::{ScreenChange, TopScreenWatch};

fn change(old: u64, new: u64) -> Option<ScreenChange> {
    Some(ScreenChange { old, new })
}

// One frame as the game drew it on 26.63: the views in draw order, each with the root control name it has.
fn frame(watch: &TopScreenWatch, views: &[(u64, &str)]) {
    for (screen, name) in views {
        if watch.note_drawn(*screen) {
            watch.set_name(*screen, name);
        }
    }
}

#[test]
fn the_top_screen_is_the_drawn_view_whose_name_is_the_top_screen_name() {
    let watch = TopScreenWatch::new();
    // The HUD and the toast are drawn after the inventory in every frame (run hm-63-r2, `[9, 11, 15, 3, 5]`).
    frame(&watch, &[(9, "chat_a"), (11, "chat_b"), (15, "inventory_screen"), (3, "hud_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("inventory_screen", false), change(0, 15));
}

#[test]
fn escape_back_to_the_hud_is_a_change_although_the_hud_was_drawn_last_all_along() {
    let watch = TopScreenWatch::new();
    frame(&watch, &[(3, "hud_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("hud_screen", false), change(0, 3));

    frame(&watch, &[(15, "inventory_screen"), (3, "hud_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("inventory_screen", false), change(3, 15));

    frame(&watch, &[(15, "inventory_screen"), (3, "hud_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("inventory_screen", false), None);

    // One frame of delay: the closing screen can still be drawn in the tick the name changes.
    frame(&watch, &[(15, "inventory_screen"), (3, "hud_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("hud_screen", false), change(15, 3));
}

#[test]
fn a_detail_after_the_name_still_matches() {
    let watch = TopScreenWatch::new();
    frame(&watch, &[(1, "world_loading_progress_screen"), (5, "toast_screen")]);
    assert_eq!(watch.tick("world_loading_progress_screen - local_world_load", false), change(0, 1));
}

#[test]
fn a_name_that_only_starts_the_same_does_not_match() {
    let watch = TopScreenWatch::new();
    frame(&watch, &[(1, "inventory_screen_pocket")]);
    assert_eq!(watch.tick("inventory_screen", false), None);
}

#[test]
fn toast_on_top_is_not_a_change() {
    let watch = TopScreenWatch::new();
    frame(&watch, &[(1, "hud_screen"), (2, "toast_screen")]);
    assert_eq!(watch.tick("hud_screen", false), change(0, 1));

    frame(&watch, &[(1, "hud_screen"), (2, "toast_screen")]);
    assert_eq!(watch.tick("toast_screen", true), None);

    frame(&watch, &[(1, "hud_screen"), (2, "toast_screen")]);
    assert_eq!(watch.tick("hud_screen", false), None);
}

#[test]
fn the_new_screen_gets_one_tick_to_be_drawn_then_the_top_is_none() {
    let watch = TopScreenWatch::new();
    frame(&watch, &[(3, "hud_screen")]);
    assert_eq!(watch.tick("hud_screen", false), change(0, 3));

    // The name changed before the first draw of the new screen: no change yet.
    frame(&watch, &[(3, "hud_screen")]);
    assert_eq!(watch.tick("pause_screen", false), None);

    // Still no drawn view with that name: no view of the top screen is drawn.
    frame(&watch, &[(3, "hud_screen")]);
    assert_eq!(watch.tick("pause_screen", false), change(3, 0));
    frame(&watch, &[(3, "hud_screen")]);
    assert_eq!(watch.tick("pause_screen", false), None);

    frame(&watch, &[(7, "pause_screen"), (3, "hud_screen")]);
    assert_eq!(watch.tick("pause_screen", false), change(0, 7));
}

#[test]
fn no_view_drawn_gives_none() {
    let watch = TopScreenWatch::new();
    assert_eq!(watch.tick("hud_screen", false), None);

    frame(&watch, &[(5, "hud_screen")]);
    assert_eq!(watch.tick("hud_screen", false), change(0, 5));

    assert_eq!(watch.tick("hud_screen", false), change(5, 0));
    assert_eq!(watch.tick("hud_screen", false), None);
}

#[test]
fn a_screen_is_asked_for_its_name_until_it_has_one_and_again_after_forget() {
    let watch = TopScreenWatch::new();
    assert!(watch.note_drawn(4));
    assert!(watch.note_drawn(4));
    watch.set_name(4, "inventory_screen");
    assert!(!watch.note_drawn(4));
    assert_eq!(watch.name_of(4).as_deref(), Some("inventory_screen"));

    watch.forget(4);
    assert_eq!(watch.name_of(4), None);
    assert!(watch.note_drawn(4));
}

#[test]
fn drawn_order_lists_the_screens_of_the_last_tick_by_their_last_draw() {
    let watch = TopScreenWatch::new();
    assert!(watch.drawn_order().is_empty());

    watch.note_drawn(4);
    watch.note_drawn(7);
    watch.note_drawn(4);
    watch.note_drawn(9);
    watch.note_drawn(4);
    watch.tick("x", false);
    assert_eq!(watch.drawn_order(), vec![7, 9, 4]);

    watch.note_drawn(2);
    watch.tick("toast_screen", true);
    assert_eq!(watch.drawn_order(), vec![2]);

    watch.tick("x", false);
    assert!(watch.drawn_order().is_empty());
}

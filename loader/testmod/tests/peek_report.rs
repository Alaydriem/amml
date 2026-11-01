use amml_testmod::PeekReport;

#[test]
fn hover_line_names_the_collection_index_stack_and_count() {
    let line = PeekReport::hover_line(0x1234, "container_items", 3, Some("minecraft:red_shulker_box"), Some(1));
    assert_eq!(
        line,
        "probe: hover controller=0x1234 collection=container_items index=3 type=minecraft:red_shulker_box count=1"
    );
}

#[test]
fn hover_line_shows_na_for_a_stack_that_could_not_be_read() {
    let line = PeekReport::hover_line(0x10, "hotbar_items", 0, None, None);
    assert!(line.ends_with("type=n/a count=n/a"));
}

#[test]
fn items_line_lists_each_slot_and_marks_an_unreadable_one() {
    let slots = [Some(0), Some(13), None];
    assert_eq!(PeekReport::items_line(Some(&slots)), "probe: shulker Items length=3 slots=[0, 13, n/a]");
}

#[test]
fn items_line_reports_a_box_with_no_items_list() {
    assert_eq!(PeekReport::items_line(None), "probe: shulker Items=n/a");
}

#[test]
fn screen_line_shows_size_and_pointer() {
    assert_eq!(
        PeekReport::screen_line(Some([427.0, 240.0]), Some([0.0, 239.5])),
        "probe: screen size=[427.0, 240.0] pointer=[0.0, 239.5]"
    );
    assert_eq!(PeekReport::screen_line(None, None), "probe: screen size=n/a pointer=n/a");
}

#[test]
fn dtor_and_after_dtor_lines_tell_whether_the_hover_was_tracked_and_drawn() {
    assert_eq!(PeekReport::dtor_line(0xabc, true), "probe: controller dtor controller=0xabc tracked=true");
    assert_eq!(PeekReport::after_dtor_line(false), "probe: first render after controller dtor drew=false");
}

#[test]
fn scroll_line_shows_the_step_the_threads_and_the_controller_tests() {
    assert_eq!(
        PeekReport::scroll_line([0.0, -1.0], true, 120, 120, Some(true), false),
        "probe: scroll x=0 y=-1 shift=true thread=120 render_thread=120 container=true hovered_controller=false"
    );
    assert!(PeekReport::scroll_line([0.5, 0.0], false, 7, 9, None, false).contains("container=n/a"));
}

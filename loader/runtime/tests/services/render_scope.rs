use amml_runtime::RenderScope;

#[test]
fn call_outside_the_dispatch_is_refused() {
    assert!(!RenderScope::allows(7));
    assert_eq!(RenderScope::current(), None);

    {
        let _frame = RenderScope::enter(7);
        assert!(RenderScope::allows(7));
        assert!(!RenderScope::allows(8));
        // Another thread has no frame, whatever this thread runs.
        std::thread::scope(|scope| {
            scope.spawn(|| assert!(!RenderScope::allows(7)));
        });
    }

    assert!(!RenderScope::allows(7));
}

#[test]
fn the_frame_zero_is_never_current() {
    let _frame = RenderScope::enter(0);
    assert!(!RenderScope::allows(0));
}

#[test]
fn a_nested_dispatch_gives_the_outer_frame_back() {
    let _outer = RenderScope::enter(1);
    {
        let _inner = RenderScope::enter(2);
        assert!(RenderScope::allows(2));
    }
    assert!(RenderScope::allows(1));
}

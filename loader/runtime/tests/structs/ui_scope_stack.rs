use amml_runtime::{UiScopeKind, UiScopeStack};

#[test]
fn the_innermost_scope_answers() {
    let mut stack = UiScopeStack::new();
    stack.push(UiScopeKind::ScreenRender { view: 1 });
    stack.push(UiScopeKind::ScreenRender { view: 2 });
    assert_eq!(stack.current(), Some(UiScopeKind::ScreenRender { view: 2 }));

    stack.pop();
    assert_eq!(stack.current(), Some(UiScopeKind::ScreenRender { view: 1 }));
    stack.pop();
    assert_eq!(stack.current(), None);
}

#[test]
fn only_the_innermost_screen_render_view_is_open() {
    let mut stack = UiScopeStack::new();
    assert!(!stack.runs_view(7));

    stack.push(UiScopeKind::ScreenRender { view: 7 });
    assert!(stack.runs_view(7));
    assert!(!stack.runs_view(8));

    stack.push(UiScopeKind::ScreenRender { view: 8 });
    assert!(!stack.runs_view(7));
}

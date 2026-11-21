use amml_runtime::{CameraScope, RenderScope};

#[test]
fn call_outside_the_camera_dispatch_is_refused() {
    assert!(!CameraScope::allows(4));
    {
        let _frame = CameraScope::enter(4);
        assert!(CameraScope::allows(4));
        assert!(!CameraScope::allows(5));
        std::thread::scope(|scope| {
            scope.spawn(|| assert!(!CameraScope::allows(4)));
        });
    }
    assert!(!CameraScope::allows(4));
}

#[test]
fn the_camera_and_render_frames_are_separate() {
    let _render = RenderScope::enter(9);

    assert!(!CameraScope::allows(9));

    let _camera = CameraScope::enter(9);
    assert!(CameraScope::allows(9));
    assert!(RenderScope::allows(9));
}

#[test]
fn the_frame_zero_is_never_a_camera_frame() {
    let _frame = CameraScope::enter(0);

    assert!(!CameraScope::allows(0));
    assert_eq!(CameraScope::current(), None);
}

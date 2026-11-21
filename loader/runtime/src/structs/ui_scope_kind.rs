/// What a running UI scope lends the table calls: for `ScreenRender`, the view whose dispatch runs now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiScopeKind {
    ScreenRender { view: u64 },
}

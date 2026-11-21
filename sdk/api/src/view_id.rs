/// The id of one view of a screen in a `ScreenRender` event. The loader answers questions about a view only while
/// the event that carries its id is being delivered; an id kept longer is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewId(pub u64);

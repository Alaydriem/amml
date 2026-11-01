use amml_i18n::text::Text;

use super::RenderJobRow;

/// What the render queue shows: the line under its title (as in "4 jobs · 2 to go · about 14 s left"), whether the
/// queue is paused, and its jobs in order.
#[derive(Debug, Clone)]
pub struct RenderQueueView {
    pub subtitle: Text,
    pub paused: bool,
    pub jobs: Vec<RenderJobRow>,
}

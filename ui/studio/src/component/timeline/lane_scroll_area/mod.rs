use amml_ui::tokens::{SpacingToken, spacing};
use egui::{Rect, Ui, pos2, vec2};

/// Paper: `54P-0`, the lane scroll area. The only part of the timeline that scrolls: one `--spacing-lane` row per
/// track, then the "Add track" row last. The caller keeps the offset.
pub struct LaneScrollArea {
    viewport: Rect,
    offset: f32,
    rows: usize,
    add_track: bool,
}

impl LaneScrollArea {
    /// Paper: the "Add track" row is 32 high.
    pub const ADD_TRACK: f32 = 32.0;

    /// The area in `viewport` scrolled down by `offset`, clamped to the content.
    pub fn new(viewport: Rect, offset: f32, rows: usize, add_track: bool) -> Self {
        let mut area = Self { viewport, offset: 0.0, rows, add_track };
        area.offset = offset.clamp(0.0, area.max_offset());
        area
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    /// How far the content can scroll: the rows and the "Add track" row (when it is on) less the viewport's height.
    pub fn max_offset(&self) -> f32 {
        let content = Self::lane() * self.rows as f32 + self.add_track_height();
        (content - self.viewport.height()).max(0.0)
    }

    /// The row of track `index`, header and lane, full width.
    pub fn row(&self, index: usize) -> Rect {
        let top = self.viewport.top() - self.offset + Self::lane() * index as f32;
        Rect::from_min_size(pos2(self.viewport.left(), top), vec2(self.viewport.width(), Self::lane()))
    }

    /// The "Add track" row, right under the last track. Off, it is 0 high.
    pub fn add_track(&self) -> Rect {
        let top = self.viewport.top() - self.offset + Self::lane() * self.rows as f32;
        Rect::from_min_size(pos2(self.viewport.left(), top), vec2(self.viewport.width(), self.add_track_height()))
    }

    /// The offset after this frame's mouse wheel over the viewport, when it changed.
    pub fn scrolled(&self, ui: &Ui) -> Option<f32> {
        let over = ui.rect_contains_pointer(self.viewport);
        let delta = ui.input(|input| input.smooth_scroll_delta.y);
        let offset = (self.offset - delta).clamp(0.0, self.max_offset());
        (over && offset != self.offset).then_some(offset)
    }

    fn add_track_height(&self) -> f32 {
        if self.add_track { Self::ADD_TRACK } else { 0.0 }
    }

    fn lane() -> f32 {
        <spacing::Lane as SpacingToken>::VALUE
    }
}

use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{LoadingKind, LoadingState};
use egui::{Context, Rect, Shape, pos2, vec2};

use crate::support::{PaintCapture, Sample};

#[test]
fn skeleton_paints_rows_raised() {
    let lane = <spacing::Lane as SpacingToken>::VALUE;
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(622.0, lane * 4.0));
    let state = LoadingState::new(LoadingKind::Skeleton { rows: 4 });
    let shapes = PaintCapture::shapes(|painter| state.paint(painter, rect));
    let blocks: Vec<_> =
        PaintCapture::rects(&shapes).into_iter().filter(|r| r.fill == color::BgRaised::VALUE).collect();
    // a thumbnail, two text bars and a trailing bar on each row
    assert_eq!(blocks.len(), 16, "{blocks:?}");
    let thumbs: Vec<_> = blocks.iter().filter(|b| b.rect.size() == vec2(80.0, 45.0)).collect();
    assert_eq!(thumbs.len(), 4);
    for (index, thumb) in thumbs.iter().enumerate() {
        let lane_centre = rect.top() + lane * (index as f32 + 0.5);
        assert!((thumb.rect.center().y - lane_centre).abs() <= 0.5, "row {index} is not `--spacing-lane` high");
    }
    assert!(blocks.iter().all(|b| rect.contains_rect(b.rect)), "a block is outside the rect");
}

#[test]
fn inline_spinner_turns() {
    let state = LoadingState::new(LoadingKind::Inline(Sample::text("Loading replays")));
    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(414.0, 30.0));
    let ctx = Context::default();
    let arc = |time: f64| {
        let shapes = PaintCapture::shapes_in(&ctx, time, |painter| state.paint(painter, rect));
        let label = shapes.iter().find_map(|shape| match shape {
            Shape::Text(text) if text.galley.text() == "Loading replays" => {
                Some(text.galley.job.sections[0].format.color)
            }
            _ => None,
        });
        assert_eq!(label, Some(color::TextDim::VALUE));
        shapes.into_iter().find_map(|shape| match shape {
            Shape::Path(path) => Some(path.points),
            _ => None,
        })
    };
    let (Some(before), Some(after)) = (arc(0.0), arc(0.3)) else { panic!("no spinner arc") };
    assert_ne!(before, after, "the spinner did not turn");
}

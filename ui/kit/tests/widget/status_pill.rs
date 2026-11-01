use amml_ui::tokens::rule::SoftFill;
use amml_ui::tokens::{ColorToken, color};
use amml_ui::widget::{PillTone, StatusPill};
use egui::{Align2, Color32, Shape, pos2};

use crate::support::{PaintCapture, Sample};

struct PillPaint;

impl PillPaint {
    // (fill, dot, text colour, pill rect)
    fn look(
        tone: PillTone,
        anchor: egui::Pos2,
        align: Align2,
    ) -> (Color32, Option<Color32>, Option<Color32>, egui::Rect) {
        let mut rect = egui::Rect::NOTHING;
        let shapes = PaintCapture::shapes(|painter| {
            rect = StatusPill::new(tone, Sample::text("Ready")).paint(painter, anchor, align)
        });
        let fill =
            PaintCapture::rects(&shapes).into_iter().find(|r| r.rect == rect).map_or(Color32::TRANSPARENT, |r| r.fill);
        let dot = shapes.iter().find_map(|s| if let Shape::Circle(c) = s { Some(c.fill) } else { None });
        let text = shapes
            .iter()
            .find_map(|s| if let Shape::Text(t) = s { Some(t.galley.job.sections[0].format.color) } else { None });
        (fill, dot, text, rect)
    }
}

#[test]
fn role_tone_uses_soft_fill_of_its_colour() {
    let role = Color32::from_rgb(0x8A, 0xA4, 0xC8);
    let (fill, dot, text, rect) = PillPaint::look(PillTone::Role(role), pos2(20.0, 40.0), Align2::LEFT_CENTER);
    assert_eq!((fill, dot, text), (SoftFill::of(role), Some(role), Some(role)));
    assert_eq!(rect.height(), 20.0);
}

#[test]
fn accent_pill_has_accent_dot_and_accent_hover_text() {
    let (fill, dot, text, _) = PillPaint::look(PillTone::Accent, pos2(20.0, 40.0), Align2::LEFT_CENTER);
    assert_eq!(
        (fill, dot, text),
        (color::AccentSoft::VALUE, Some(color::Accent::VALUE), Some(color::AccentHover::VALUE))
    );
}

#[test]
fn named_tones_follow_the_sheet() {
    let tones = [
        (PillTone::Dim, SoftFill::of(color::TextDim::VALUE), color::TextDim::VALUE),
        (PillTone::Faint, SoftFill::of(color::TextFaint::VALUE), color::TextFaint::VALUE),
        (PillTone::Success, SoftFill::of(color::Success::VALUE), color::Success::VALUE),
        (PillTone::Warning, color::WarningSoft::VALUE, color::Warning::VALUE),
        (PillTone::Danger, color::DangerSoft::VALUE, color::Danger::VALUE),
    ];
    for (tone, fill, text) in tones {
        let (f, dot, t, _) = PillPaint::look(tone, pos2(20.0, 40.0), Align2::LEFT_CENTER);
        assert_eq!((f, dot, t), (fill, Some(text), Some(text)), "{tone:?}");
    }
}

#[test]
fn right_anchor_ends_at_the_anchor() {
    let anchor = pos2(300.0, 40.0);
    let (_, _, _, rect) = PillPaint::look(PillTone::Dim, anchor, Align2::RIGHT_CENTER);
    assert_eq!(rect.right_center(), anchor);
}

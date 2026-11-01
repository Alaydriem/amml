use std::f32::consts::PI;

use egui::{Id, Ui, pos2};

use super::{BarRows, TimelineSamples, ViewerSamples};
use crate::component::easing::{EasingFamily, EasingPopover, EasingPopoverView, EasingPreset};
use crate::gallery::GalleryState;
use crate::id::StudioUiId;
use crate::tokens::selector::TrackColor;

/// Paper: Menus + Dialogs, `Easing popover` (`7J6-0` custom bezier, `8MQ-0` named curve, 3 keys). The saved preset
/// menu (`95U-0`) is an amml-ui `Menu` the host opens (Plan 08).
pub struct EasingPopoverPage;

impl EasingPopoverPage {
    /// Paper: the popover sheets are 900 wide.
    pub const WIDTH: f32 = 900.0;
    const STEPS: usize = 32;
    // Paper `7J6-0`: the custom curve's handles, (119.7, 119.7) and (110.2, 0) in a 190 box
    const HANDLES: [[f32; 2]; 2] = [[119.7 / 190.0, 70.3 / 190.0], [110.2 / 190.0, 1.0]];
    // Paper: the 24 cards in grid order, with each one's family
    const NAMES: [(&'static str, EasingFamily); 24] = [
        ("Smooth", EasingFamily::Other),
        ("In Quad", EasingFamily::In),
        ("Out Quad", EasingFamily::Out),
        ("In Out Quad", EasingFamily::InOut),
        ("In Cubic", EasingFamily::In),
        ("Out Cubic", EasingFamily::Out),
        ("In Out Cubic", EasingFamily::InOut),
        ("In Quart", EasingFamily::In),
        ("Out Quart", EasingFamily::Out),
        ("In Out Quart", EasingFamily::InOut),
        ("In Quint", EasingFamily::In),
        ("Out Quint", EasingFamily::Out),
        ("In Out Quint", EasingFamily::InOut),
        ("In Sine", EasingFamily::In),
        ("Out Sine", EasingFamily::Out),
        ("In Out Sine", EasingFamily::InOut),
        ("In Expo", EasingFamily::In),
        ("Out Expo", EasingFamily::Out),
        ("In Out Expo", EasingFamily::InOut),
        ("In Circ", EasingFamily::In),
        ("Out Circ", EasingFamily::Out),
        ("In Out Circ", EasingFamily::InOut),
        ("In Back", EasingFamily::In),
        ("Out Back", EasingFamily::Out),
    ];

    pub fn show(ui: &mut Ui, _state: &mut GalleryState) {
        BarRows::page(ui, |ui| {
            ViewerSamples::row(ui, "CUSTOM BEZIER", |ui| {
                let [first, second] = Self::HANDLES;
                let view = EasingPopoverView {
                    subtitle: TimelineSamples::words("Camera key · applies to 1 selected key · now: Custom bezier"),
                    track: TrackColor::Camera,
                    samples: Self::samples(&|t| Self::bezier(first, second, t)),
                    handles: Some([pos2(first[0], first[1]), pos2(second[0], second[1])]),
                    family: EasingFamily::All,
                    presets: Self::presets(),
                    selected: None,
                    saved: Self::saved(2),
                    selected_saved: None,
                };
                EasingPopover::new(&view).show(ui);
            });
            ViewerSamples::row(ui, "NAMED CURVE · 3 KEYS", |ui| {
                let view = EasingPopoverView {
                    subtitle: TimelineSamples::words("Camera key · applies to 3 selected keys · now: In Out Cubic"),
                    track: TrackColor::Camera,
                    samples: Self::samples(&|t| Self::named("In Out Cubic", t)),
                    handles: None,
                    family: EasingFamily::All,
                    presets: Self::presets(),
                    selected: Some(6),
                    saved: Self::saved(3),
                    selected_saved: Some(0),
                };
                EasingPopover::new(&view).id(Id::unique(StudioUiId::EasingFamily).with(1)).show(ui);
            });
        });
    }

    fn presets() -> Vec<EasingPreset> {
        Self::NAMES
            .iter()
            .map(|(name, family)| EasingPreset {
                name: TimelineSamples::words(name),
                samples: Self::samples(&|t| Self::named(name, t)),
                family: *family,
            })
            .collect()
    }

    // Paper's saved presets: Soft landing, Whip pan and Custom 3, as their bezier handles
    fn saved(count: usize) -> Vec<EasingPreset> {
        let curves = [
            ("Soft landing", [6.0 / 30.0, 0.7], [0.4, 1.0]),
            ("Whip pan", [0.6, 0.0], [14.0 / 30.0, 1.0]),
            ("Custom 3", [0.4, 0.0], [0.6, 1.0]),
        ];
        curves
            .iter()
            .take(count)
            .map(|(name, first, second)| EasingPreset {
                name: TimelineSamples::words(name),
                samples: Self::samples(&|t| Self::bezier(*first, *second, t)),
                family: EasingFamily::Other,
            })
            .collect()
    }

    fn samples(ease: &dyn Fn(f32) -> f32) -> Vec<f32> {
        (0..=Self::STEPS).map(|step| ease(step as f32 / Self::STEPS as f32)).collect()
    }

    // the y of a unit cubic bezier with handles `first` and `second` at x = `t`, found by bisection on its x
    fn bezier(first: [f32; 2], second: [f32; 2], t: f32) -> f32 {
        let along =
            |a: f32, b: f32, s: f32| 3.0 * (1.0 - s) * (1.0 - s) * s * a + 3.0 * (1.0 - s) * s * s * b + s * s * s;
        let (mut low, mut high) = (0.0_f32, 1.0_f32);
        for _ in 0..24 {
            let mid = (low + high) / 2.0;
            if along(first[0], second[0], mid) < t { low = mid } else { high = mid }
        }
        along(first[1], second[1], (low + high) / 2.0)
    }

    // the standard easing curves by name
    fn named(name: &str, t: f32) -> f32 {
        let kind = name.rsplit(' ').next().unwrap_or("");
        let in_out = name.starts_with("In Out");
        let into = !in_out && name.starts_with("In");
        let power = |p: i32| {
            if in_out {
                if t < 0.5 { 2f32.powi(p - 1) * t.powi(p) } else { 1.0 - (-2.0 * t + 2.0).powi(p) / 2.0 }
            } else if into {
                t.powi(p)
            } else {
                1.0 - (1.0 - t).powi(p)
            }
        };
        match kind {
            "Quad" => power(2),
            "Cubic" => power(3),
            "Quart" => power(4),
            "Quint" => power(5),
            "Sine" if in_out => -((PI * t).cos() - 1.0) / 2.0,
            "Sine" if into => 1.0 - (t * PI / 2.0).cos(),
            "Sine" => (t * PI / 2.0).sin(),
            "Expo" if in_out => {
                if t <= 0.0 || t >= 1.0 {
                    t
                } else if t < 0.5 {
                    2f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            "Expo" if into => {
                if t <= 0.0 {
                    0.0
                } else {
                    2f32.powf(10.0 * t - 10.0)
                }
            }
            "Expo" => {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - 2f32.powf(-10.0 * t)
                }
            }
            "Circ" if in_out => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            "Circ" if into => 1.0 - (1.0 - t * t).sqrt(),
            "Circ" => (1.0 - (t - 1.0).powi(2)).sqrt(),
            "Back" if into => 2.70158 * t * t * t - 1.70158 * t * t,
            "Back" => 1.0 + 2.70158 * (t - 1.0).powi(3) + 1.70158 * (t - 1.0).powi(2),
            _ => t * t * (3.0 - 2.0 * t),
        }
    }
}

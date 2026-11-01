use std::sync::Arc;

use amml_i18n::message::Message;
use amml_i18n::service::{LocaleSetting, LocalizationService};
use amml_studio_ui::message::StudioUiMessage;
use amml_studio_ui::text::StudioUiPacks;
use amml_ui::AmmlUiMessage;
use amml_ui::font::{FontLoader, FontSource};
use amml_ui::text::{AmmlUiPacks, UiLocale};
use egui::{Context, Event, PointerButton, Pos2, RawInput, Rect, Shape, Ui, vec2};

// drives a component through headless frames with the built-in fonts, and keeps what each frame painted
pub struct UiProbe {
    ctx: Context,
    frame: u32,
    width: f32,
    height: f32,
}

impl UiProbe {
    pub fn new(width: f32) -> Self {
        Self::sized(width, 400.0)
    }

    // a probe whose screen is `width` × `height`
    pub fn sized(width: f32, height: f32) -> Self {
        let ctx = Context::default();
        FontLoader::install(&ctx, FontSource::BuiltIn);
        Self { ctx, frame: 0, width, height }
    }

    // a probe whose amml-studio-ui and amml-ui words are the en_XA pseudo-locale (longer, accented)
    pub fn pseudo(width: f32, height: f32) -> Self {
        let probe = Self::sized(width, height);
        let service = LocalizationService::new(vec![
            (StudioUiMessage::DOMAIN, StudioUiPacks::SOURCE),
            (AmmlUiMessage::DOMAIN, AmmlUiPacks::SOURCE),
        ]);
        UiLocale::set(&probe.ctx, Arc::new(service.load(&LocaleSetting::from_code("en_XA"))));
        probe
    }

    // runs one frame and returns what `show` returned and what the frame painted
    pub fn frame<T>(&mut self, events: Vec<Event>, mut show: impl FnMut(&mut Ui) -> T) -> (T, Vec<Shape>) {
        self.frame += 1;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(self.width, self.height))),
            time: Some(1.0 + f64::from(self.frame) / 60.0),
            events,
            ..Default::default()
        };
        let mut out = None;
        let mut output = self.ctx.run_ui(input, |ui| out = Some(show(ui)));
        output.textures_delta.clear();
        let shapes = output.shapes.into_iter().map(|clipped| clipped.shape).collect();
        match out {
            Some(value) => (value, shapes),
            None => panic!("run_ui did not call the component"),
        }
    }

    // two quiet frames (fonts settle), then the shapes of the second
    pub fn shapes<T>(&mut self, mut show: impl FnMut(&mut Ui) -> T) -> Vec<Shape> {
        self.frame(Vec::new(), &mut show);
        self.frame(Vec::new(), &mut show).1
    }

    // move, press and release at `at`; returns the value of the release frame
    pub fn click<T>(&mut self, at: Pos2, mut show: impl FnMut(&mut Ui) -> T) -> T {
        self.frame(vec![Event::PointerMoved(at)], &mut show);
        let press = Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        };
        self.frame(vec![press], &mut show);
        let release = Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        };
        self.frame(vec![release], &mut show).0
    }

    // rests the pointer on `at` with no tooltip delay; returns what the last frame painted, tooltips included
    pub fn hover<T>(&mut self, at: Pos2, mut show: impl FnMut(&mut Ui) -> T) -> Vec<Shape> {
        self.ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
        self.frame(vec![Event::PointerMoved(at)], &mut show);
        self.frame(Vec::new(), &mut show);
        self.frame(Vec::new(), &mut show).1
    }

    // move, press and release the right button at `at`; returns the value of the release frame
    pub fn secondary_click<T>(&mut self, at: Pos2, mut show: impl FnMut(&mut Ui) -> T) -> T {
        self.frame(vec![Event::PointerMoved(at)], &mut show);
        let button = |pressed| Event::PointerButton {
            pos: at,
            button: PointerButton::Secondary,
            pressed,
            modifiers: Default::default(),
        };
        self.frame(vec![button(true)], &mut show);
        self.frame(vec![button(false)], &mut show).0
    }

    // every rect shape, nested shapes included
    pub fn rects(shapes: &[Shape]) -> Vec<egui::epaint::RectShape> {
        let mut out = Vec::new();
        for shape in shapes {
            match shape {
                Shape::Rect(rect) => out.push(rect.clone()),
                Shape::Vec(inner) => out.extend(Self::rects(inner)),
                _ => {}
            }
        }
        out
    }

    // the painted text of every shape that is not an icon glyph (private-use characters)
    pub fn words(shapes: &[Shape]) -> Vec<String> {
        Self::texts(shapes)
            .into_iter()
            .filter(|text| !text.chars().all(|c| ('\u{E000}'..='\u{F8FF}').contains(&c)))
            .collect()
    }

    // one frame with `key` pressed
    pub fn key(key: egui::Key) -> Event {
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }
    }

    // press at `from`, move to `to` in four steps, release; returns the value of every frame
    pub fn drag<T>(&mut self, from: Pos2, to: Pos2, mut show: impl FnMut(&mut Ui) -> T) -> Vec<T> {
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        let mut values = vec![self.frame(vec![Event::PointerMoved(from)], &mut show).0];
        values.push(self.frame(vec![button(from, true)], &mut show).0);
        for step in 1..=4 {
            let at = from + (to - from) * (step as f32 / 4.0);
            values.push(self.frame(vec![Event::PointerMoved(at)], &mut show).0);
        }
        values.push(self.frame(vec![button(to, false)], &mut show).0);
        values
    }

    // turn the wheel by `delta` points with the pointer at `at`, then let the smooth scroll settle
    pub fn wheel<T>(&mut self, at: Pos2, delta: egui::Vec2, mut show: impl FnMut(&mut Ui) -> T) {
        let wheel = Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta,
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Wheel,
            modifiers: Default::default(),
        };
        self.frame(vec![Event::PointerMoved(at), wheel], &mut show);
        for _ in 0..120 {
            self.frame(Vec::new(), &mut show);
        }
    }

    // the painted rect of the first text shape that reads `text`
    pub fn text_rect(shapes: &[Shape], text: &str) -> Option<Rect> {
        shapes.iter().find_map(|shape| match shape {
            Shape::Text(painted) if painted.galley.text() == text => {
                Some(painted.galley.rect.translate(painted.pos.to_vec2()))
            }
            _ => None,
        })
    }

    // every painted text
    pub fn texts(shapes: &[Shape]) -> Vec<String> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Text(painted) => Some(painted.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }
}

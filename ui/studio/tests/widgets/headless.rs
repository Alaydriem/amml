use egui::{Context, Event, PointerButton, Pos2, RawInput, Rect, Response, Ui, pos2, vec2};

// drives one widget through headless frames: a move, a press and a release over a point
pub struct Headless {
    ctx: Context,
    frame: u32,
}

impl Headless {
    pub fn new() -> Self {
        Self { ctx: Context::default(), frame: 0 }
    }

    fn input(&mut self, events: Vec<Event>) -> RawInput {
        self.frame += 1;
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(800.0, 600.0))),
            time: Some(1.0 + f64::from(self.frame) / 60.0),
            events,
            ..Default::default()
        }
    }

    // runs one frame and returns what the widget returned
    pub fn frame<T>(&mut self, events: Vec<Event>, mut show: impl FnMut(&mut Ui) -> T) -> T {
        let input = self.input(events);
        let mut out = None;
        let mut output = self.ctx.run_ui(input, |ui| {
            out = Some(show(ui));
        });
        output.textures_delta.clear();
        match out {
            Some(value) => value,
            None => panic!("run_ui did not call the widget"),
        }
    }

    // the rect of a widget shown with `show`
    pub fn rect(&mut self, mut show: impl FnMut(&mut Ui) -> Response) -> Rect {
        self.frame(Vec::new(), |ui| show(ui).rect)
    }

    // move, press, release at `at`; returns the value of the release frame
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
        self.frame(vec![release], &mut show)
    }

    pub fn centre(rect: Rect) -> Pos2 {
        pos2(rect.center().x, rect.center().y)
    }
}

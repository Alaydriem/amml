use amml_ui::motion::Motion;
use egui::{Context, RawInput};

#[test]
fn turn_moves_with_time() {
    let ctx = Context::default();
    let mut turns = Vec::new();
    for time in [1.0, 1.25] {
        let input = RawInput { time: Some(time), ..Default::default() };
        let mut output = ctx.run_ui(input, |ui| turns.push(Motion::turn(ui.ctx())));
        output.textures_delta.clear();
    }
    let step = (turns[1] - turns[0]).rem_euclid(1.0);
    assert!((step - 0.2).abs() < 0.01, "{turns:?}");
    assert!(turns.iter().all(|turn| (0.0..1.0).contains(turn)));
}

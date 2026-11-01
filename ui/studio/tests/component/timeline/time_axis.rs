use amml_studio_ui::component::timeline::TimeAxis;

#[test]
fn time_axis_round_trips() {
    for seconds_per_point in [0.05 / 1268.0, 0.01, 1.0, 10_800.0 / 1280.0] {
        let axis = TimeAxis { start: 12.5, seconds_per_point, left: 222.0 };
        for offset in [0.0, 0.001, 0.37, 2.0] {
            let seconds = 12.5 + offset * seconds_per_point * 1000.0;
            let back = axis.seconds_at(axis.x_of(seconds));
            let tolerance = 1e-6_f64.max(seconds_per_point * 1e-3);
            assert!((back - seconds).abs() <= tolerance, "{seconds} came back as {back} at {seconds_per_point}");
        }
    }
}

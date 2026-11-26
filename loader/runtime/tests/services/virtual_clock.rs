use amml_runtime::{TimerReading, VirtualClock};

const SIM: usize = 0;
const REAL: usize = 1;
const TPS: f32 = 20.0;

#[test]
fn steps_sum_to_exact_game_time() {
    let mut clock = VirtualClock::new();
    // Odd step sizes, none a whole number of ticks, many times over: a float tick count would drift here.
    let steps: [i64; 6] = [16_667, 33_333, 1, 49_999, 7, 8_333];
    let mut total = 0i64;
    let mut ticks = [0i64; 2];
    let mut last = TimerReading { ticks: 0, alpha: 0.0 };
    for round in 0..20_000 {
        let micros = steps[round % steps.len()];
        clock.step_us(micros);
        total += micros;
        assert!(clock.apply());
        last = clock.read(SIM, TPS);
        ticks[SIM] += i64::from(last.ticks);
        // The second timer of the frame sees the same game time and takes no second step.
        assert!(!clock.apply());
        ticks[REAL] += i64::from(clock.read(REAL, TPS).ticks);
    }

    assert_eq!(clock.stepped_us(), total);
    // Every whole tick of the stepped time was handed out once, to each timer.
    assert_eq!(ticks, [total * 20 / 1_000_000; 2]);
    // The alpha is the rest of the stepped time, as a fraction of a tick.
    let rest = (total * 20 % 1_000_000) as f64 / 1_000_000.0;
    assert_eq!(last.alpha, rest as f32);
}

#[test]
fn alpha_is_the_fraction_of_a_tick() {
    let mut clock = VirtualClock::new();
    clock.step_us(25_000);
    clock.apply();
    assert_eq!(clock.read(SIM, TPS), TimerReading { ticks: 0, alpha: 0.5 });

    clock.step_us(50_000);
    clock.apply();
    assert_eq!(clock.read(SIM, TPS), TimerReading { ticks: 1, alpha: 0.5 });

    // A frame with no step hands out no tick and keeps the alpha.
    assert!(!clock.apply());
    assert_eq!(clock.read(SIM, TPS), TimerReading { ticks: 0, alpha: 0.5 });

    // A timer read for the first time gets every tick of the hold so far.
    assert_eq!(clock.read(REAL, TPS), TimerReading { ticks: 1, alpha: 0.5 });
    // A timer of another rate reads the same game time at its own rate.
    assert_eq!(clock.read(2, 10.0), TimerReading { ticks: 0, alpha: 0.75 });
}

#[test]
fn steps_queued_in_one_frame_reach_the_timers_together() {
    let mut clock = VirtualClock::new();
    clock.step_us(30_000);
    clock.step_us(20_000);
    assert_eq!(clock.stepped_us(), 0);

    assert!(clock.apply());
    assert_eq!(clock.stepped_us(), 50_000);
    assert_eq!(clock.read(SIM, TPS), TimerReading { ticks: 1, alpha: 0.0 });
}

#[test]
fn a_rate_drop_never_gives_negative_ticks() {
    let mut clock = VirtualClock::new();
    clock.step_us(1_000_000);
    clock.apply();
    assert_eq!(clock.read(SIM, TPS).ticks, 20);

    // The rate halves: the stepped second is 10 ticks at the new rate, below the 20 already handed out.
    assert_eq!(clock.read(SIM, 10.0).ticks, 0);
    // Later steps hand out the ticks of the new rate from there.
    clock.step_us(100_000);
    clock.apply();
    assert_eq!(clock.read(SIM, 10.0).ticks, 1);
}

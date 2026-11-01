use amml_testmod::Throttle;

#[test]
fn the_passing_calls_are_the_first_the_101st_and_the_201st() {
    let throttle = Throttle::new(100);
    let passed: Vec<u32> = (1..=250).filter(|_| throttle.hit()).collect();
    assert_eq!(passed, vec![1, 101, 201]);
}

#[test]
fn a_throttle_of_zero_passes_every_call() {
    let throttle = Throttle::new(0);
    assert!((0..5).all(|_| throttle.hit()));
}

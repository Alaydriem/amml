use amml_testmod::TickCounter;

#[test]
fn only_the_call_that_reaches_the_target_reports_and_the_count_stops_there() {
    let counter = TickCounter::new(3);
    let reports: Vec<bool> = (0..6).map(|_| counter.tick()).collect();
    assert_eq!(reports, vec![false, false, true, false, false, false]);
    assert_eq!(counter.count(), 3);
}

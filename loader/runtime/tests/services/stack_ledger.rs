use amml_runtime::StackLedger;

#[test]
fn held_and_lent_stacks_pass() {
    let ledger = StackLedger::new();
    ledger.lend(7, 0x2000);

    assert!(ledger.accepts(7, 0x1000, &[0x1000]));
    assert!(ledger.accepts(7, 0x2000, &[0x1000]));
    assert!(!ledger.accepts(7, 0x3000, &[0x1000]));
    assert!(!ledger.accepts(7, 0, &[0]));
}

#[test]
fn a_lent_stack_from_an_ended_scope_is_refused() {
    let ledger = StackLedger::new();
    ledger.lend(7, 0x2000);

    assert!(!ledger.accepts(8, 0x2000, &[]));
    ledger.lend(8, 0x3000);
    assert!(!ledger.accepts(7, 0x2000, &[]));
    assert!(ledger.accepts(8, 0x3000, &[]));
}

#[test]
fn a_stack_lent_twice_in_one_scope_is_recorded_once() {
    let ledger = StackLedger::new();
    ledger.lend(7, 0x2000);
    ledger.lend(7, 0x2000);

    assert_eq!(ledger.len(), 1);
}

#[test]
fn forgetting_a_scope_refuses_its_lent_stacks_and_keeps_held_ones() {
    let ledger = StackLedger::new();
    ledger.lend(7, 0x2000);

    ledger.forget(7);

    assert!(!ledger.accepts(7, 0x2000, &[]));
    assert!(ledger.accepts(7, 0x1000, &[0x1000]));
    ledger.lend(7, 0x3000);
    assert!(ledger.accepts(7, 0x3000, &[]));
}

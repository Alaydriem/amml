use amml_abi::{AmmlInputBlocks, AmmlLeaseMode, AmmlLeaseState};
use amml_runtime::{LeaseBook, LeaseNotice, LeaseRefusal, LeaseTerms, NoticeKind, ResourceId};

const A: ResourceId = ResourceId(1);
const B: ResourceId = ResourceId(2);
const C: ResourceId = ResourceId(3);

fn terms(mode: AmmlLeaseMode, blocks: AmmlInputBlocks, passthrough: Vec<u16>, preempt: bool) -> LeaseTerms {
    LeaseTerms { mode, blocks, passthrough, preempt }
}

fn shared() -> LeaseTerms {
    terms(AmmlLeaseMode::SHARED, AmmlInputBlocks::KEYS, Vec::new(), false)
}

fn exclusive() -> LeaseTerms {
    terms(AmmlLeaseMode::EXCLUSIVE, AmmlInputBlocks::FULL, Vec::new(), false)
}

fn preempt() -> LeaseTerms {
    terms(AmmlLeaseMode::EXCLUSIVE, AmmlInputBlocks::FULL, Vec::new(), true)
}

fn ranking(book: &LeaseBook, ids: &[&str]) {
    book.set_ranking(9, ids.iter().map(|id| id.to_string()).collect()).unwrap();
}

fn coexist(book: &LeaseBook, pairs: &[(&str, &str)]) {
    book.set_coexist(9, pairs.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()).unwrap();
}

#[test]
fn shared_leases_live_together() {
    let book = LeaseBook::new();
    assert_eq!(book.acquire(A, 1, "@a/x", shared()), Ok(Vec::new()));
    assert_eq!(book.acquire(B, 2, "@b/y", shared()), Ok(Vec::new()));
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
}

#[test]
fn an_exclusive_lease_suspends_the_shared_holders() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    let notices = book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
}

#[test]
fn the_shared_holders_resume_when_the_exclusive_lease_ends() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    book.acquire(C, 3, "@c/z", shared()).unwrap();
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(
        book.end(B),
        vec![
            LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed },
            LeaseNotice { owner: 3, lease: C, kind: NoticeKind::Resumed },
        ]
    );
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(C), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_coexisting_shared_holder_stays_active_beside_an_exclusive_lease() {
    let book = LeaseBook::new();
    coexist(&book, &[("@b/y", "@a/x")]);
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    book.acquire(C, 3, "@c/z", shared()).unwrap();
    let notices = book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 3, lease: C, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(C), AmmlLeaseState::SUSPENDED);
}

#[test]
fn a_shared_lease_is_granted_beside_a_coexisting_exclusive_lease() {
    let book = LeaseBook::new();
    coexist(&book, &[("@a/x", "@b/y")]);
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(book.acquire(B, 2, "@b/y", shared()), Ok(Vec::new()));
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
    assert_eq!(book.acquire(C, 3, "@c/z", shared()), Err(LeaseRefusal::ExclusiveHeld));
}

#[test]
fn an_exclusive_lease_refused_by_an_exclusive_holder_suspends_no_shared_holder() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    coexist(&book, &[("@a/x", "@b/y")]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(book.acquire(C, 3, "@c/z", exclusive()), Err(LeaseRefusal::ExclusiveHeld));
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_new_coexist_pair_resumes_the_suspended_shared_holder() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    let notices = book.set_coexist(9, vec![("@a/x".to_string(), "@b/y".to_string())]).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_removed_coexist_pair_suspends_the_shared_holder() {
    let book = LeaseBook::new();
    coexist(&book, &[("@a/x", "@b/y")]);
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    let notices = book.set_coexist(9, Vec::new()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.end(B), vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
}

#[test]
fn an_outranked_exclusive_that_resumes_suspends_the_shared_holders() {
    let book = LeaseBook::new();
    ranking(&book, &["@a/x", "@b/y"]);
    coexist(&book, &[("@c/z", "@a/x")]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(book.acquire(C, 3, "@c/z", shared()), Ok(Vec::new()));
    assert_eq!(
        book.end(A),
        vec![
            LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Resumed },
            LeaseNotice { owner: 3, lease: C, kind: NoticeKind::Suspended },
        ]
    );
    assert_eq!(book.state(C), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.end(B), vec![LeaseNotice { owner: 3, lease: C, kind: NoticeKind::Resumed }]);
}

#[test]
fn only_the_ranking_owner_may_set_the_coexist_pairs() {
    let book = LeaseBook::new();
    book.set_ranking(1, Vec::new()).unwrap();
    assert_eq!(book.set_coexist(2, Vec::new()), Err(LeaseRefusal::RankingOwned));
    assert_eq!(book.set_coexist(1, Vec::new()), Ok(Vec::new()));
    book.end_owner(1);
    assert_eq!(book.set_coexist(2, Vec::new()), Ok(Vec::new()));
    assert_eq!(book.set_ranking(1, Vec::new()), Err(LeaseRefusal::RankingOwned));
}

#[test]
fn a_shared_lease_is_refused_while_an_exclusive_one_is_active() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(book.acquire(B, 2, "@b/y", shared()), Err(LeaseRefusal::ExclusiveHeld));
}

#[test]
fn a_preempt_lease_suspends_the_others_until_it_ends() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    let notices = book.acquire(B, 2, "@b/y", preempt()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);

    let notices = book.end(B);
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(B), AmmlLeaseState::LOST);
}

#[test]
fn a_shared_lease_taken_during_a_preempt_starts_suspended() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", preempt()).unwrap();
    assert_eq!(book.acquire(B, 2, "@b/y", shared()), Ok(Vec::new()));
    assert_eq!(book.state(B), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.blocks_now(), AmmlInputBlocks::FULL);
    assert_eq!(book.end(A), vec![LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Resumed }]);
}

#[test]
fn a_second_preempt_lease_is_refused() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", preempt()).unwrap();
    assert_eq!(book.acquire(B, 2, "@b/y", preempt()), Err(LeaseRefusal::PreemptHeld));
}

#[test]
fn a_higher_ranked_exclusive_suspends_a_lower_one() {
    let book = LeaseBook::new();
    ranking(&book, &["@a/x", "@b/y"]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    let notices = book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(B), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.blocks_now(), AmmlInputBlocks::FULL);
}

#[test]
fn a_lease_outranked_by_a_higher_ranked_mod_resumes_when_the_winner_ends() {
    let book = LeaseBook::new();
    ranking(&book, &["@a/x", "@b/y"]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(book.end(A), vec![LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Resumed }]);
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
}

#[test]
fn ending_a_preempt_does_not_resume_a_lease_whose_winner_still_holds() {
    let book = LeaseBook::new();
    ranking(&book, &["@a/x", "@b/y"]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    let notices = book.acquire(C, 3, "@c/z", preempt()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Suspended }]);
    assert_eq!(book.end(C), vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
    assert_eq!(book.state(B), AmmlLeaseState::SUSPENDED);
}

#[test]
fn an_outranked_lease_stays_suspended_while_a_higher_winner_holds() {
    let book = LeaseBook::new();
    ranking(&book, &["@c/z", "@b/y", "@a/x"]);
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(
        book.acquire(C, 3, "@c/z", exclusive()).unwrap(),
        vec![LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Suspended }]
    );
    assert_eq!(book.end(B), Vec::new());
    assert_eq!(book.state(A), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.end(C), vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_winner_that_ends_during_a_preempt_leaves_its_loser_to_the_preempt() {
    let book = LeaseBook::new();
    ranking(&book, &["@a/x", "@b/y"]);
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    book.acquire(C, 3, "@c/z", preempt()).unwrap();
    assert_eq!(book.end(A), Vec::new());
    assert_eq!(book.state(B), AmmlLeaseState::SUSPENDED);
    assert_eq!(book.end(C), vec![LeaseNotice { owner: 2, lease: B, kind: NoticeKind::Resumed }]);
}

#[test]
fn a_lower_ranked_exclusive_is_refused() {
    let book = LeaseBook::new();
    book.set_ranking(9, vec!["@a/x".to_string(), "@b/y".to_string()]).unwrap();
    book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(book.acquire(B, 2, "@b/y", exclusive()), Err(LeaseRefusal::ExclusiveHeld));
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_ranked_mod_beats_an_unranked_one() {
    let book = LeaseBook::new();
    book.set_ranking(9, vec!["@a/x".to_string()]).unwrap();
    book.acquire(C, 3, "@c/z", exclusive()).unwrap();
    let notices = book.acquire(A, 1, "@a/x", exclusive()).unwrap();
    assert_eq!(notices, vec![LeaseNotice { owner: 3, lease: C, kind: NoticeKind::Suspended }]);
    assert_eq!(book.state(A), AmmlLeaseState::ACTIVE);
}

#[test]
fn between_two_unranked_mods_the_first_exclusive_wins() {
    let book = LeaseBook::new();
    book.set_ranking(9, vec!["@a/x".to_string()]).unwrap();
    book.acquire(B, 2, "@b/y", exclusive()).unwrap();
    assert_eq!(book.acquire(C, 3, "@c/z", exclusive()), Err(LeaseRefusal::ExclusiveHeld));
    assert_eq!(book.state(B), AmmlLeaseState::ACTIVE);
}

#[test]
fn only_the_ranking_owner_may_change_it() {
    let book = LeaseBook::new();
    assert_eq!(book.set_ranking(1, vec!["@a/x".to_string()]), Ok(()));
    assert_eq!(book.set_ranking(2, vec!["@b/y".to_string()]), Err(LeaseRefusal::RankingOwned));
    assert_eq!(book.set_ranking(1, vec!["@b/y".to_string()]), Ok(()));
    book.end_owner(1);
    assert_eq!(book.set_ranking(2, vec!["@b/y".to_string()]), Ok(()));
}

#[test]
fn blocks_are_the_union_of_active_leases_and_passthrough_lets_a_key_through() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", terms(AmmlLeaseMode::SHARED, AmmlInputBlocks::KEYS, vec![0x1B], false)).unwrap();
    book.acquire(B, 2, "@b/y", terms(AmmlLeaseMode::SHARED, AmmlInputBlocks::LOOK, Vec::new(), false)).unwrap();
    assert_eq!(book.blocks_now(), AmmlInputBlocks(AmmlInputBlocks::KEYS.0 | AmmlInputBlocks::LOOK.0));
    assert!(book.passes(0x1B));
    assert!(!book.passes(0x57));
}

#[test]
fn a_suspended_lease_blocks_nothing() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", terms(AmmlLeaseMode::SHARED, AmmlInputBlocks::KEYS, Vec::new(), false)).unwrap();
    book.acquire(B, 2, "@b/y", terms(AmmlLeaseMode::SHARED, AmmlInputBlocks::LOOK, vec![0x57], true)).unwrap();
    assert_eq!(book.blocks_now(), AmmlInputBlocks::LOOK);
    assert!(book.passes(0x57));
}

#[test]
fn ending_an_owner_resumes_the_others_and_tells_the_owner_nothing() {
    let book = LeaseBook::new();
    book.acquire(A, 1, "@a/x", shared()).unwrap();
    book.acquire(B, 2, "@b/y", preempt()).unwrap();
    book.acquire(C, 2, "@b/y", shared()).unwrap();
    assert_eq!(book.end_owner(2), vec![LeaseNotice { owner: 1, lease: A, kind: NoticeKind::Resumed }]);
    assert_eq!(book.owner_of(B), None);
    assert_eq!(book.owner_of(C), None);
    assert_eq!(book.owner_of(A), Some(1));
}

#[test]
fn an_unknown_lease_is_lost() {
    let book = LeaseBook::new();
    assert_eq!(book.state(ResourceId(12345)), AmmlLeaseState::LOST);
    assert_eq!(book.end(ResourceId(12345)), Vec::new());
}

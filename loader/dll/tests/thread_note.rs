use amml::ThreadNote;
use amml_abi::AmmlThread;

#[test]
fn the_first_thread_and_each_change_are_noted() {
    let note = ThreadNote::new();

    let seen: Vec<bool> =
        [AmmlThread::GAME, AmmlThread::GAME, AmmlThread::RENDER, AmmlThread::RENDER, AmmlThread::GAME]
            .into_iter()
            .map(|thread| note.changed(thread))
            .collect();

    assert_eq!(seen, [true, false, true, false, true]);
}

#[test]
fn changes_past_the_cap_are_not_noted() {
    let note = ThreadNote::new();
    let threads = [AmmlThread::GAME, AmmlThread::RENDER];

    let noted = (0..20).filter(|turn| note.changed(threads[turn % 2])).count();

    assert_eq!(noted, usize::from(ThreadNote::MAX_LINES));
}

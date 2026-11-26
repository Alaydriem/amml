use amml_abi::AmmlBlockPos;
use amml_runtime::{BlockChange, BlockChangeQueue};

fn change(x: i32) -> BlockChange {
    BlockChange { dimension: 0, pos: AmmlBlockPos { x, y: 64, z: 0 }, layer: 0, new_block: 0x10, old_block: 0x20 }
}

fn taken(queue: &BlockChangeQueue, epoch: u64) -> (Vec<i32>, u64) {
    let mut xs = Vec::new();
    let dropped = queue.take(epoch, |change| xs.push(change.pos.x));
    (xs, dropped)
}

#[test]
fn off_thread_change_is_raised_at_the_next_frame() {
    let queue = BlockChangeQueue::new();
    let worker = std::thread::scope(|scope| {
        scope.spawn(|| (queue.push(1, change(1)), queue.push(1, change(2)))).join().unwrap()
    });
    assert_eq!(worker, (true, true));

    let (first, dropped) = taken(&queue, 1);

    assert_eq!((first, dropped), (vec![1, 2], 0));
    // The frame took them: the next frame has none.
    assert_eq!(taken(&queue, 1), (Vec::new(), 0));
}

#[test]
fn queued_change_is_dropped_after_world_left() {
    let queue = BlockChangeQueue::new();
    queue.push(1, change(1));

    queue.clear();

    assert_eq!(taken(&queue, 1), (Vec::new(), 0));

    // A change that raced past the clear in the old epoch is not given out in the next world.
    queue.push(1, change(2));
    assert_eq!(taken(&queue, 2), (Vec::new(), 0));
}

#[test]
fn full_queue_drops_and_counts() {
    let queue = BlockChangeQueue::new();
    for x in 0..i32::try_from(BlockChangeQueue::CAPACITY).unwrap() {
        assert!(queue.push(1, change(x)));
    }

    assert!(!queue.push(1, change(-1)));
    assert!(!queue.push(1, change(-2)));
    let (xs, dropped) = taken(&queue, 1);

    assert_eq!(xs.len(), BlockChangeQueue::CAPACITY);
    assert_eq!(xs.first(), Some(&0));
    assert_eq!(dropped, 2);
    // The count is given once; the emptied queue takes changes again.
    assert!(queue.push(1, change(5)));
    assert_eq!(taken(&queue, 1), (vec![5], 0));
}

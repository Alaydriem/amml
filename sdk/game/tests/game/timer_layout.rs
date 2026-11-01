use amml_game::TimerLayout;

use crate::support::GameMemory;

#[test]
fn reads_and_writes_the_used_fields_at_the_game_offsets() {
    let mut memory = GameMemory::new(0x80);
    memory.put::<f32>(0x00, 20.0); // mTicksPerSecond

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let timer = unsafe { &mut *memory.ptr().cast::<TimerLayout>() };
    assert_eq!(timer.ticks_per_second, 20.0);

    timer.ticks = 3;
    timer.alpha = 0.25;
    timer.passed_time = 0.75;

    assert_eq!(memory.get::<i32>(0x04), 3); // mTicks
    assert_eq!(memory.get::<f32>(0x08), 0.25); // mAlpha
    assert_eq!(memory.get::<f32>(0x10), 0.75); // mPassedTime
}

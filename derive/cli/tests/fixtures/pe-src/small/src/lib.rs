use std::hint::black_box;

#[inline(never)]
fn mix(values: &mut [u32; 64], seed: u32) -> u32 {
    for (i, v) in values.iter_mut().enumerate() {
        *v = black_box(seed.wrapping_mul(i as u32 + 1));
    }
    values.iter().fold(0, |a, b| a.rotate_left(3) ^ *b)
}

#[unsafe(no_mangle)]
pub extern "C" fn amml_probe(seed: u32) -> u32 {
    let mut values = [0u32; 64];
    mix(&mut values, seed)
}

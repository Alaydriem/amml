use amml_runtime::{MaterialBytes, MaterialChoice, MaterialPatch, MaterialWrite};

const MATERIAL: usize = 0x5000;
const OTHER_MATERIAL: usize = 0x6000;
const MOD_A: usize = 1;
const MOD_B: usize = 2;
// The bytes the game's wireframe material has: opaque, depth write on.
const GAME: MaterialBytes = MaterialBytes { enable_blend: 0, blend_source: 2, blend_destination: 1, depth_write: 1 };

fn write(address: usize, bytes: MaterialBytes) -> MaterialWrite {
    MaterialWrite { address, bytes }
}

#[test]
fn restore_writes_the_kept_bytes_once() {
    let patch = MaterialPatch::new();
    let plan = patch.prepare(MOD_A, true, Some((MATERIAL, GAME)));
    assert_eq!(plan.writes, [write(MATERIAL, MaterialPatch::PATCHED)]);
    assert_eq!(plan.choice, MaterialChoice::Wireframe);
    assert!(patch.is_applied());

    patch.request_restore();

    assert_eq!(patch.take_restore(Some(MATERIAL)), Some(write(MATERIAL, GAME)));
    assert_eq!(patch.take_restore(Some(MATERIAL)), None);
    // Nothing is kept any more, so a later request writes nothing.
    patch.request_restore();
    assert_eq!(patch.take_restore(Some(MATERIAL)), None);
    assert!(!patch.is_applied());
}

#[test]
fn switch_restores_before_the_other_material() {
    let patch = MaterialPatch::new();
    patch.prepare(MOD_A, true, Some((MATERIAL, GAME)));

    // The first draw after the switch out of Vibrant Visuals: the restore comes first, then the name tag material.
    let switched = patch.prepare(MOD_A, false, Some((MATERIAL, MaterialPatch::PATCHED)));
    let next = patch.prepare(MOD_A, false, Some((MATERIAL, GAME)));

    assert_eq!(switched.writes, [write(MATERIAL, GAME)]);
    assert_eq!(switched.choice, MaterialChoice::NameTag);
    assert!(next.writes.is_empty());
    assert!(!patch.is_applied());
}

#[test]
fn disable_of_the_last_mod_restores_the_material() {
    let patch = MaterialPatch::new();
    patch.prepare(MOD_A, true, Some((MATERIAL, GAME)));
    patch.prepare(MOD_B, true, Some((MATERIAL, MaterialPatch::PATCHED)));

    patch.disable_owner(MOD_A);
    assert!(!patch.restore_due());
    patch.disable_owner(MOD_B);

    assert!(patch.restore_due());
    assert_eq!(patch.take_restore(Some(MATERIAL)), Some(write(MATERIAL, GAME)));
}

#[test]
fn a_kept_material_that_is_gone_is_never_written() {
    let patch = MaterialPatch::new();
    patch.prepare(MOD_A, true, Some((MATERIAL, GAME)));
    patch.request_restore();

    // The renderer of a left world is gone; the running one has another material.
    assert_eq!(patch.take_restore(Some(OTHER_MATERIAL)), None);

    // A new patch keeps the bytes of the new material.
    patch.prepare(MOD_A, true, Some((OTHER_MATERIAL, GAME)));
    patch.request_restore();
    assert_eq!(patch.take_restore(None), None);
}

#[test]
fn a_due_restore_is_written_before_a_new_patch() {
    let patch = MaterialPatch::new();
    patch.prepare(MOD_A, true, Some((MATERIAL, GAME)));
    patch.request_restore();

    let plan = patch.prepare(MOD_A, true, Some((MATERIAL, MaterialPatch::PATCHED)));

    assert_eq!(plan.writes, [write(MATERIAL, GAME), write(MATERIAL, MaterialPatch::PATCHED)]);
    // The bytes kept for the next restore are the game's, not the patched ones.
    patch.request_restore();
    assert_eq!(patch.take_restore(Some(MATERIAL)), Some(write(MATERIAL, GAME)));
}

use crate::support::{Fixture, Record, RecordingListener};
use amml_msvc_fixture::fx_listener_drive;

#[test]
fn cpp_drives_a_rust_object_through_its_vtable_and_deletes_it() {
    let mut record = Record::default();
    let object = RecordingListener::allocate(Fixture::heap(), &mut record).unwrap();
    // SAFETY: `object` is a heap-allocated Rust object with a Listener-shaped vtable; C++ deletes it.
    let result = unsafe { fx_listener_drive(object) };
    assert_eq!(result, 21);
    assert_eq!(record.name.as_deref(), Some("abc"));
    assert!(record.destroyed_with_delete_flag);
    assert!(record.state_dropped);
}

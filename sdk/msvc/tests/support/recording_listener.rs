use super::{ListenerVtable, Record};
use amml_msvc::{CallbackGuard, MsvcError, MsvcHeap, MsvcString, RustObject};
use std::ffi::c_void;

/// State of the Rust object that C++ sees as a `Listener`.
pub struct RecordingListener {
    heap: MsvcHeap,
    record: *mut Record,
}

type ListenerObject = RustObject<ListenerVtable, RecordingListener>;

static VTABLE: ListenerVtable = ListenerVtable {
    destructor: RecordingListener::destroy,
    on_value: RecordingListener::on_value,
    on_name: RecordingListener::on_name,
};

impl Drop for RecordingListener {
    fn drop(&mut self) {
        // SAFETY: the record outlives the object.
        unsafe { (*self.record).state_dropped = true };
    }
}

impl RecordingListener {
    /// Allocates the listener on the game heap. `record` must outlive the object.
    pub fn allocate(heap: MsvcHeap, record: *mut Record) -> Result<*mut c_void, MsvcError> {
        // SAFETY: the fixture heap is the real operator new/delete.
        let object = unsafe { ListenerObject::allocate_in(&heap, &VTABLE, Self { heap, record }) };
        object.map(|object| object.cast())
    }

    unsafe extern "C" fn destroy(this: *mut c_void, flags: u32) -> *mut c_void {
        CallbackGuard::run(|| {
            let this = this.cast::<ListenerObject>();
            // SAFETY: C++ calls the destructor on the live object it was given.
            let state = unsafe { &*ListenerObject::state(this) };
            let (heap, record) = (state.heap, state.record);
            // SAFETY: the record outlives the object.
            unsafe {
                (*record).destroyed_with_delete_flag = flags & 1 != 0;
            }
            // SAFETY: the object came from `allocate_in` with this heap and C++ does not use it again.
            unsafe { ListenerObject::destroy(this, flags, &heap) }
        })
    }

    unsafe extern "C" fn on_value(_this: *mut c_void, value: i32) -> i32 {
        CallbackGuard::run(|| value + 1)
    }

    unsafe extern "C" fn on_name(this: *mut c_void, name: *const MsvcString) {
        CallbackGuard::run(|| {
            // SAFETY: C++ passes a live `const std::string&` and the object is live.
            let (name, state) = unsafe { (&*name, &*ListenerObject::state(this.cast())) };
            // SAFETY: the record outlives the object.
            unsafe { (*state.record).name = name.to_str().ok().map(str::to_owned) };
        })
    }
}

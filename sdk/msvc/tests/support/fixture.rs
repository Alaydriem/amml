use amml_msvc::{MsvcHeap, SharedPtr};
use std::mem::MaybeUninit;

pub struct Fixture;

impl Fixture {
    pub fn heap() -> MsvcHeap {
        // SAFETY: the probes return the addresses of the fixture's operator new and operator delete.
        let (new, delete) = unsafe { (amml_msvc_fixture::fx_operator_new(), amml_msvc_fixture::fx_operator_delete()) };
        MsvcHeap::new(new, delete)
    }

    pub fn thing(k: i32) -> super::Thing {
        super::Thing { k }
    }

    /// A C++ `std::shared_ptr<Tracked>` holding `value`, made by the fixture.
    pub fn make_shared(value: i32) -> SharedPtr<i32> {
        let mut slot = MaybeUninit::<SharedPtr<i32>>::uninit();
        // SAFETY: the slot has the size of a shared_ptr, and the probe constructs one in it.
        unsafe { amml_msvc_fixture::fx_shared_make(value, slot.as_mut_ptr().cast()) };
        // SAFETY: the probe initialised the slot.
        unsafe { slot.assume_init() }
    }
}

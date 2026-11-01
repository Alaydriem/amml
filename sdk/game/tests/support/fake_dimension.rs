use std::ffi::c_void;

use amml_msvc::RustObject;

use super::dimension_vtable::DimensionVtable;

static VTABLE: DimensionVtable = DimensionVtable { reserved: 0, get_dimension_id: FakeDimension::get_dimension_id };

/// A fake `Dimension` whose id function returns a fixed value through the hidden return pointer.
pub struct FakeDimension {
    object: RustObject<DimensionVtable, i32>,
}

impl FakeDimension {
    pub const ID_SLOT: u64 = 1;

    pub fn new(id: i32) -> Self {
        Self { object: RustObject::new(&VTABLE, id) }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut self.object).cast()
    }

    extern "C" fn get_dimension_id(this: *mut c_void, out: *mut i32) -> *mut i32 {
        // SAFETY: `this` is the RustObject made by `FakeDimension::new`, and `out` is the caller's return buffer.
        unsafe { *out = *RustObject::<DimensionVtable, i32>::state(this.cast()) };
        out
    }
}

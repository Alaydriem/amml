use std::ffi::c_void;

use amml_msvc::RustObject;

use super::options_vtable::OptionsVtable;

static OPTIONS_VTABLE: OptionsVtable = OptionsVtable { reserved: 0, get_hide_hud: FakeOptions::get_hide_hud };

/// A fake `IOptionRegistry` whose HUD is hidden or not.
pub struct FakeOptions {
    object: RustObject<OptionsVtable, bool>,
}

impl FakeOptions {
    pub const HIDE_HUD_SLOT: u64 = 1;

    pub fn new(hide_hud: bool) -> Self {
        Self { object: RustObject::new(&OPTIONS_VTABLE, hide_hud) }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        (&raw mut self.object).cast()
    }

    extern "C" fn get_hide_hud(this: *mut c_void) -> u8 {
        // SAFETY: `this` is the RustObject made by `FakeOptions::new`.
        u8::from(unsafe { *RustObject::<OptionsVtable, bool>::state(this.cast()) })
    }
}

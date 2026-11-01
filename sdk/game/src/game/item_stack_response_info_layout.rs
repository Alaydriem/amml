use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcVec;

use crate::game::ItemStackResponseContainerInfoLayout;

/// Layout mirror of the game's `ItemStackResponseInfo` (48 bytes), the answer to one item stack request.
#[repr(C)]
pub struct ItemStackResponseInfoLayout {
    pub result: u8,                    // 0x00 mResult (ItemStackNetResult: Success 0)
    _request_id_vtable: *const c_void, // 0x08 mClientRequestId (a TypedClientNetId, vptr)
    pub client_request_id: i32,        // 0x10 mClientRequestId.mRawId
    pub containers: MsvcVec<ItemStackResponseContainerInfoLayout>, // 0x18 mContainers
}

const _: () = assert!(offset_of!(ItemStackResponseInfoLayout, client_request_id) == 0x10);
const _: () = assert!(offset_of!(ItemStackResponseInfoLayout, containers) == 0x18);
const _: () = assert!(size_of::<ItemStackResponseInfoLayout>() == 0x30);

use amml_msvc::MsvcString;

/// `SlotData`, the argument of `ContainerManagerController::handleSwap`: a `std::string` collection name, then an
/// `int` index at +20h. The game reads it through a pointer and does not keep it.
#[repr(C)]
pub struct ContainerSlotData {
    pub name: MsvcString,
    pub index: i32,
}

const _: () = assert!(std::mem::offset_of!(ContainerSlotData, index) == 0x20);

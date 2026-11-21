use amml_msvc::MsvcVec;

use crate::CameraPresetLayout;

/// The head of the game's `CameraPresets`, which `Level::getCameraPresets` gives by reference: the preset vector at
/// +0. Read from the exe on 26.63 and 70.26: the `CameraInstructionPacket` handler passes the result of
/// `getCameraPresets` (slot 1Dh of the level) to the camera instruction function, which reads the vector begin at +0
/// and end at +8 of it. The members after the vector are not read, so the mirror stops there.
#[repr(C)]
pub struct CameraPresetsLayout {
    pub presets: MsvcVec<CameraPresetLayout>, // 0x00 mPresets
}

use amml_msvc::MsvcString;

/// The game's `CameraPreset` (130h bytes): the name (the text of its identifier) first, then members the loader does
/// not read. `CameraPreset.size` checks the size. Read from the exe on 26.63 and 70.26: the camera instruction function
/// reads the name of the preset at the set index as an MSVC string at +0 (`[preset+10h]` the size, `[preset+18h]`
/// against 10h for the inline buffer).
#[repr(C)]
pub struct CameraPresetLayout {
    pub name: MsvcString, // 0x000 mName
    _rest: [u64; 34],     // 0x020 mInheritFrom .. mStartingRot
}

impl CameraPresetLayout {
    /// The size the `CameraPreset.size` entry must have.
    pub const SIZE: u64 = size_of::<CameraPresetLayout>() as u64;
}

const _: () = assert!(size_of::<CameraPresetLayout>() == 0x130);

use crate::{CameraInstructionLayout, CameraPresetLayout, HandlerDetours};

/// The game entries the free camera calls: `MinecraftPackets::createPacket`, the instruction offset of a
/// `CameraInstructionPacket`, the `CameraInstructionPacket` handler slot, the `getCameraPresets` slot of the level and
/// the level of an actor (`Actor.mLevel`).
#[derive(Clone, Copy, Debug)]
pub struct FreeCameraCalls {
    pub create_packet: usize,
    pub instruction_at: usize,
    pub camera_slot: usize,
    pub presets_slot: usize,
    pub level_at: usize,
}

impl FreeCameraCalls {
    /// The instruction offset of a `CameraInstructionPacket`, when the instruction entries fit
    /// `CameraInstructionLayout`; else the entry that is missing or does not fit.
    pub fn instruction_at(lookup: &impl Fn(&str) -> Option<u64>) -> Result<usize, &'static str> {
        const SPLINE: &str = "CameraInstruction.mSpline";
        const AT: &str = "CameraInstructionPacket.mCameraInstruction";
        const SIZE: &str = "CameraInstructionPacket.size";
        let spline = lookup(SPLINE).ok_or(SPLINE)?;
        let at = lookup(AT).ok_or(AT)?;
        let size = lookup(SIZE).ok_or(SIZE)?;
        if spline != CameraInstructionLayout::SPLINE {
            return Err(SPLINE);
        }
        if !CameraInstructionLayout::fits(spline, at, size) {
            return Err(SIZE);
        }
        usize::try_from(at).map_err(|_| AT)
    }

    /// Every call of the free camera, when each entry resolved and the instruction and preset entries fit their
    /// mirrors; else the entry that is missing or does not fit.
    pub fn from_lookup(lookup: &impl Fn(&str) -> Option<u64>) -> Result<Self, &'static str> {
        const CREATE: &str = "MinecraftPackets.createPacket";
        const PRESETS: &str = "Level.getCameraPresets";
        const PRESET_SIZE: &str = "CameraPreset.size";
        const LEVEL: &str = "Actor.mLevel";
        let value = |name: &'static str| lookup(name).and_then(|value| usize::try_from(value).ok()).ok_or(name);
        let instruction_at = Self::instruction_at(lookup)?;
        if lookup(PRESET_SIZE).ok_or(PRESET_SIZE)? != CameraPresetLayout::SIZE {
            return Err(PRESET_SIZE);
        }
        Ok(Self {
            create_packet: value(CREATE).and_then(|address| if address == 0 { Err(CREATE) } else { Ok(address) })?,
            instruction_at,
            camera_slot: value(HandlerDetours::CAMERA)?,
            presets_slot: value(PRESETS)?,
            level_at: value(LEVEL)?,
        })
    }
}

use amml_abi::AmmlPose;

/// The pose of the current camera frame and the mod that set it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HeldPose {
    pub(crate) owner: usize,
    pub(crate) pose: AmmlPose,
}

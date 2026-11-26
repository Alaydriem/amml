/// Why the pose book refuses a pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PoseRefusal {
    #[error("the pose names another frame than the current camera frame")]
    OtherFrame,
    #[error("another mod set the pose of this frame first")]
    Taken,
}

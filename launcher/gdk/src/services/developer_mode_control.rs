use crate::GdkError;

// The Developer Mode switch the command line may offer to turn on.
pub trait DeveloperModeControl {
    fn is_on(&self) -> Result<bool, GdkError>;

    // Turns it on after the player said yes. Returns whether it is on afterwards.
    fn enable_with_consent(&self) -> Result<bool, GdkError>;
}

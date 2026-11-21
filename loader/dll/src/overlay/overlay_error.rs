use thiserror::Error;

/// Why the overlay hooks did not go in.
#[derive(Debug, Error)]
pub enum OverlayError {
    #[error("the DXGI probe failed at {step}: {source}")]
    Probe { step: &'static str, source: windows::core::Error },
    #[error("the hook of {entry} was refused with status {status}")]
    Hook { entry: &'static str, status: i32 },
    #[error("the overlay was installed already")]
    AlreadyInstalled,
}

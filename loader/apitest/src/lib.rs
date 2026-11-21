//! The end-to-end test mod of the stable mod API, built on `amml-api` only. It proves events, the
//! registry, logging, the client areas, input injection, the game window and the overlay (drawing,
//! screenshots, the capture stream) in the running game: the in-game task passes on its `apitest ok`
//! line.

#![deny(unsafe_code)]

pub mod api_test_mod;
pub mod check_phase;
pub mod check_run;
pub mod frame_cost;
pub mod input_checks;
pub mod overlay_checks;
pub mod overlay_phase;
pub mod overlay_run;
pub mod overlay_tools;
pub mod progress;
pub mod provided_echo;
pub mod step;
pub mod window_checks;

pub use api_test_mod::ApiTestMod;
pub use check_phase::CheckPhase;
pub use check_run::CheckRun;
pub use frame_cost::FrameCost;
pub use input_checks::InputChecks;
pub use overlay_checks::OverlayChecks;
pub use overlay_phase::OverlayPhase;
pub use overlay_run::OverlayRun;
pub use overlay_tools::OverlayTools;
pub use progress::Progress;
pub use provided_echo::{ECHO, ProvidedEcho};
pub use step::Step;
pub use window_checks::WindowChecks;

amml_api::amml_mod!(ApiTestMod);

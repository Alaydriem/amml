use amml_abi::{AmmlEvent, FramePresent};
use amml_data::{AdapterCatalog, NeedKind};

use super::{DxgiProbe, OverlayError, OverlayShared, PresentHooks, ProbeTargets, SurfaceCounts};
use crate::LoaderState;

/// The loader's overlay: the DXGI and D3D12 hooks, the bind of the game's swap chain and window, and the present and
/// resize events. One per process.
///
/// The window events and areas depend on it: the window is attached by the overlay bind, so both are offered only
/// when the overlay hooks are in.
pub struct OverlayService;

impl OverlayService {
    /// The area of the draw surface and the overlay table's present functions.
    pub const SURFACE_AREA: &'static str = "overlay.surface";

    /// Probes the DXGI and D3D12 function addresses with a hidden window of its own and hooks them. Logs `overlay
    /// hooks installed` or `overlay hooks not installed`; offers the overlay and window events and areas only on
    /// success. Not for use under the OS loader lock: it makes a window and devices.
    pub fn install(state: &'static LoaderState) -> bool {
        match DxgiProbe::targets() {
            Ok(targets) => Self::install_targets(state, &targets),
            Err(error) => {
                curia::warn!("overlay hooks not installed", { error: error.to_string() });
                false
            }
        }
    }

    /// Hooks the functions at `targets`, as `install` does after its probe.
    pub fn install_targets(state: &'static LoaderState, targets: &ProbeTargets) -> bool {
        let shared = OverlayShared::global();
        if !shared.claim(state) {
            curia::warn!("overlay hooks not installed", { error: OverlayError::AlreadyInstalled.to_string() });
            return false;
        }
        match PresentHooks::install(state, targets) {
            Ok(queue_hooked) => {
                shared.arm();
                Self::offer(state);
                state.window().offer();
                curia::info!("overlay hooks installed", { queue_hook: queue_hooked });
                true
            }
            Err(error) => {
                curia::warn!("overlay hooks not installed", { error: error.to_string() });
                false
            }
        }
    }

    /// The outermost presents of the bound swap chain so far.
    pub fn present_count() -> u64 {
        OverlayShared::global().present_count()
    }

    /// The draw surfaces made so far, and the state saves for them.
    pub fn surface_counts() -> SurfaceCounts {
        OverlayShared::global().cost().counts()
    }

    // The needs with shape `overlay` in the catalog, and `FramePresent` up to the version that takes the surface on
    // demand.
    fn offer(state: &LoaderState) {
        for need in AdapterCatalog::needs() {
            if !need.shapes.iter().any(|shape| shape.name == AdapterCatalog::OVERLAY_SHAPE) {
                continue;
            }
            match need.kind {
                NeedKind::Event => state.events().offer(need.id, need.version),
                NeedKind::Area => state.areas().offer(need.id, need.version),
            }
        }
        state.events().offer(FramePresent::ID, FramePresent::ON_DEMAND_VERSION);
    }
}

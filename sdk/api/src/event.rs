use amml_abi::{
    AmmlEvent, AmmlEventHeader, BlockBreakContinue, BlockBreakStart, CameraFov, ChatMessageAdded, ClientTick,
    ContainerSlotHover, ContainerSlotUnhover, DimensionChanging, FramePresent, FramePresented, FrameUpdate, GameTick,
    GamepadPoll, IdleScreenOpen, InjectedInputReleased, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended,
    InventoryRequestDone, ItemHovertext, ItemUse, ItemUseOnBlock, JoinServerResult, JoinStarted, KeyInput,
    LocatorBarIconDraw, LookDelta, ModsLoaded, MouseGrab, MouseInput, PlayerAttack, PlayerListUpdate, PlayerMoveInput,
    PlayerSkinChanged, ProviderGone, ProviderReady, RenderDeviceReset, ScreenChanged, ScreenClosing, ScreenRender,
    ScreenshotTaken, ScreenshotWritten, Shutdown, SurfaceLost, SwapChainResize, SwapChainResized, TexturesReloaded,
    UiPointer, UiScroll, WindowChanged, WindowChar, WindowCursor, WindowFocus, WindowPointer, WindowSize, WorldJoined,
    WorldLeft,
};

/// An event a mod can listen to.
///
/// # Safety
/// `Self` is `#[repr(C)]`, starts with an `AmmlEventHeader`, and is the struct the loader delivers
/// for `ID` at `VERSION`. A listener reads the loader's bytes as `Self`.
#[allow(unsafe_code)]
pub unsafe trait Event: Sized + 'static {
    /// The event id given to the loader.
    const ID: &'static str;
    /// The layout version of the struct.
    const VERSION: u32;
    /// Whether a listener may cancel the event (`Cancel::cancel`). On an event that cannot be
    /// cancelled the loader ignores the flag, and on one that can, a later listener cannot undo it.
    const CANCELLABLE: bool = false;

    /// The header the loader wrote: phase, origin, whether it is cancelled and whether a mod
    /// injected it.
    fn header(&self) -> &AmmlEventHeader {
        // SAFETY: by the trait contract `Self` starts with an `AmmlEventHeader`; the borrow of `self`
        // covers it.
        #[allow(unsafe_code)]
        unsafe {
            &*std::ptr::from_ref(self).cast::<AmmlEventHeader>()
        }
    }

    /// Whether an earlier listener (or the loader, for input a lease blocks) cancelled the event.
    fn is_cancelled(&self) -> bool {
        self.header().cancelled != 0
    }

    /// Whether a mod injected the event through an input lease rather than the game raising it.
    fn is_injected(&self) -> bool {
        self.header().injected != 0
    }
}

macro_rules! abi_events {
    ($($event:ty),* $(,)?) => {
        $(
            // SAFETY: every `amml_abi::events` struct is `#[repr(C)]` and starts with `header`.
            #[allow(unsafe_code)]
            unsafe impl Event for $event {
                const ID: &'static str = <$event as AmmlEvent>::ID;
                const VERSION: u32 = <$event as AmmlEvent>::VERSION;
                const CANCELLABLE: bool = <$event as AmmlEvent>::CANCELLABLE;
            }
        )*
    };
}

abi_events!(
    ClientTick,
    FrameUpdate,
    GameTick,
    GamepadPoll,
    InjectedInputReleased,
    InputLeaseLost,
    InputLeaseResumed,
    InputLeaseSuspended,
    JoinServerResult,
    JoinStarted,
    KeyInput,
    LookDelta,
    ModsLoaded,
    MouseGrab,
    MouseInput,
    PlayerMoveInput,
    ProviderGone,
    ProviderReady,
    Shutdown,
    WorldJoined,
    WorldLeft,
    WindowPointer,
    WindowChar,
    WindowFocus,
    WindowSize,
    WindowChanged,
    WindowCursor,
    FramePresent,
    FramePresented,
    SwapChainResize,
    SwapChainResized,
    SurfaceLost,
    RenderDeviceReset,
    ScreenshotTaken,
    ScreenshotWritten,
    CameraFov,
    DimensionChanging,
    IdleScreenOpen,
    BlockBreakStart,
    BlockBreakContinue,
    ItemUse,
    PlayerAttack,
    ItemUseOnBlock,
    ScreenRender,
    ScreenClosing,
    ScreenChanged,
    ContainerSlotHover,
    ContainerSlotUnhover,
    UiPointer,
    UiScroll,
    ItemHovertext,
    InventoryRequestDone,
    PlayerListUpdate,
    PlayerSkinChanged,
    LocatorBarIconDraw,
    ChatMessageAdded,
    TexturesReloaded,
);

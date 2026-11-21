#![forbid(unsafe_code)]

pub mod services;
pub mod structs;

/// Why the loader counts an entry as not for this build; the deriver's `entry_is` judgement.
pub use amml_derive::LayoutRefusal;

pub use services::{
    BiomeNameBook, BlockChangeQueue, BlockTypeBook, BufferMap, ButtonTable, CacheFile, CacheSource, CameraMath,
    CameraScope, CaptureRing, ClipBook, ColumnBlocks, ColumnWalk, DeriveSource, EventBus, FocusHold, FreeCameraBook,
    Grade, HandlerCapture, HookChain, HostResolver, InjectionQueue, InlineIconBook, ItemDrawBook, KeyTable, LeaseBook,
    LeviLaminaSource, LightBook, LightCell, MaterialPatch, MeshBook, ModRegistry, PointerPairs, PointerUpGate,
    PoseBook, ProviderRegistry, RenderScope, RequestBook, ResolverChain, ResolverSource, ResourceStore, RestoreBook,
    ScreenBook, ShapeBook, ShotBook, StackIdentity, StackLedger, StackRequestEncoder, SurfaceColumn, SurfaceRule,
    SymbolLookup, TextureBook, TopScreenWatch, WaypointTags, WorldTransform,
};
pub use structs::{
    AmmlVersion, BlockChange, CallOutcome, CameraBasis, CameraFrameGuard, CameraKindNames, CapturePoint,
    CapturedHandler, ChainChange, ChainError, ClipKind, ClipStep, CollectionNames, DispatchReport, EmitterEntry,
    EntryTable, FocusMessage, FrameInjection, FreeCameraRefusal, FreeCameraRelease, GlyphPage, GradeReport, GradeRow,
    GradeVerdict, HeaderStamp, HookEntry, IconKey, IconRefusal, ImageLayout, InjectedOp, ItemDrawKey, ItemDrawRefusal,
    KeyVerdict, LeaseNotice, LeaseRefusal, LeaseTerms, LightChange, LightPoint, LightRefusal, ListenError, ListenerId,
    MaterialBytes, MaterialChoice, MaterialPlan, MaterialWrite, MeshFree, MeshKey, MeshRefusal, ModRegistryError,
    ModScan, ModSpec, NoticeKind, OriginalValue, OriginalWrite, PanelNames, PoseRefusal, ProtocolValues, ProvideError,
    ReleaseOutcome, RenderFrameGuard, RequestAction, RequestDone, RequestMove, RequestRefusal, RequestSlot,
    RequestStep, RequestTick, ResourceError, ResourceId, ResourceKind, RestoreFlag, SavedWaypoint, ScreenChange,
    ScreenKindTable, ShapeKind, ShapeRefusal, ShapeRequest, ShapeSend, ShotRequest, SkippedMod, SlotRef, SlotRelease,
    SourceNote, SourceOutcome, SurfaceHit, TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind,
    TextureTarget, TextureWork, UiScopeKind, UiScopeStack, VvPlan, WalkBlock, WalkedColumn, WaypointWrite, WorldEpoch,
    WorldPlacement,
};

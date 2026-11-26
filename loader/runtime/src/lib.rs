#![forbid(unsafe_code)]

pub mod services;
pub mod structs;

/// Why the loader counts an entry as not for this build; the deriver's `entry_is` judgement.
pub use amml_derive::LayoutRefusal;

pub use services::{
    BiomeNameBook, BlockChangeQueue, BlockTagWriter, BlockTypeBook, BufferMap, BuildReentry, ButtonTable, CacheFile,
    CacheSource, CacheWindow, CameraMath, CameraScope, CaptureRing, CellSource, ChunkCacheSwitch, ClipBook,
    ColumnBlocks, ColumnWalk, DeriveSource, EventBus, FocusHold, FocusReplayBook, FreeCameraBook, GhostEntityBook,
    Grade, GroupBook, HandlerCapture, HideBook, HookChain, HostResolver, InjectionQueue, InlineIconBook, ItemDrawBook,
    KeyTable, KindsScan, LeaseBook, LeviLaminaSource, LightBook, LightCell, LightDiff, MaterialPatch, MeshBook,
    MeshPost, ModRegistry, PointerPairs, PointerUpGate, PoseBook, ProviderRegistry, RenderScope, RequestBook,
    ResolverChain, ResolverSource, ResourceStore, RestoreBook, ScreenBook, ShapeBook, ShotBook, SkullTransform,
    StackIdentity, StackLedger, StackRequestEncoder, SurfaceColumn, SurfaceRule, SymbolLookup, TextureBook,
    TimeHoldBook, TopScreenWatch, TransientOptionBook, VirtualClock, WaypointTags, WireConnections, WorldTransform,
};
pub use structs::{
    ActionScope, AmmlVersion, BlockChange, BlockTagError, BlockTagValue, CallOutcome, CameraBasis, CameraFrameGuard,
    CameraKindNames, CapturePoint, CapturedHandler, ChainChange, ChainError, ClipKind, ClipStep, CollectionNames,
    CreativeEntry, DispatchReport, EmitterEntry, EntryTable, FocusMessage, FrameInjection, FreeCameraRefusal,
    FreeCameraRelease, GhostEntity, GhostEntityKey, GhostEntityRefusal, GlyphPage, GradeReport, GradeRow, GradeVerdict,
    GroupKey, GroupRefusal, HeaderStamp, HookEntry, IconKey, IconRefusal, ImageLayout, InjectedOp, ItemDrawKey,
    ItemDrawRefusal, KeyVerdict, KindsOutcome, KindsScanned, LeaseNotice, LeaseRefusal, LeaseTerms, LightChange,
    LightPoint, LightRefusal, ListenError, ListenerId, MaterialBytes, MaterialChoice, MaterialPlan, MaterialWrite,
    MeshFree, MeshKey, MeshKind, MeshRefusal, ModRegistryError, ModScan, ModSpec, NoticeKind, OptionRestore,
    OriginalValue, OriginalWrite, PanelNames, PoseRefusal, ProtocolValues, ProvideError, ReleaseOutcome,
    RenderFrameGuard, RequestAction, RequestDone, RequestMove, RequestRefusal, RequestSlot, RequestStep, RequestTick,
    ResourceError, ResourceId, ResourceKind, RestoreFlag, SavedWaypoint, ScreenChange, ScreenKindTable, ShapeKind,
    ShapeRefusal, ShapeRequest, ShapeSend, ShotRequest, SkippedMod, SkullFace, SlotRef, SlotRelease, SourceNote,
    SourceOutcome, SurfaceHit, TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind,
    TextureTarget, TextureWork, TimeHoldRefusal, TimeReleaseReason, TimerReading, TransientHold,
    TransientOptionRefusal, UiScopeKind, UiScopeStack, VvPlan, WalkBlock, WalkedColumn, WaypointWrite, WindowCell,
    WorldEpoch, WorldPlacement,
};

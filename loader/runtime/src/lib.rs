#![forbid(unsafe_code)]

pub mod services;
pub mod structs;

/// Why the loader counts an entry as not for this build; the deriver's `entry_is` judgement.
pub use amml_derive::LayoutRefusal;

pub use services::{
    BufferMap, ButtonTable, CacheFile, CacheSource, CaptureRing, DeriveSource, EventBus, FocusHold, Grade, HookChain,
    HostResolver, InjectionQueue, InlineIconBook, ItemDrawBook, KeyTable, LeaseBook, LeviLaminaSource, LightBook,
    LightCell, ModRegistry, PointerPairs, PointerUpGate, ProviderRegistry, RequestBook, ResolverChain, ResolverSource,
    ResourceStore, RestoreBook, ScreenBook, ShotBook, StackIdentity, StackLedger, StackRequestEncoder, SymbolLookup,
    TextureBook, TopScreenWatch, WaypointTags,
};
pub use structs::{
    AmmlVersion, CallOutcome, CapturePoint, ChainChange, ChainError, CollectionNames, DispatchReport, EmitterEntry,
    EntryTable, FocusMessage, FrameInjection, GlyphPage, GradeReport, GradeRow, GradeVerdict, HeaderStamp, HookEntry,
    IconKey, IconRefusal, ImageLayout, InjectedOp, ItemDrawKey, ItemDrawRefusal, KeyVerdict, LeaseNotice, LeaseRefusal,
    LeaseTerms, LightChange, LightPoint, LightRefusal, ListenError, ListenerId, ModRegistryError, ModScan, ModSpec,
    NoticeKind, OriginalValue, OriginalWrite, PanelNames, ProtocolValues, ProvideError, ReleaseOutcome, RequestAction,
    RequestDone, RequestMove, RequestRefusal, RequestSlot, RequestStep, RequestTick, ResourceError, ResourceId,
    ResourceKind, RestoreFlag, SavedWaypoint, ScreenChange, ScreenKindTable, ShotRequest, SkippedMod, SlotRef,
    SlotRelease, SourceNote, SourceOutcome, TextureJob, TextureKey, TextureRefusal, TextureRelease, TextureSourceKind,
    TextureTarget, TextureWork, UiScopeKind, UiScopeStack, VvPlan, WaypointWrite, WorldEpoch,
};

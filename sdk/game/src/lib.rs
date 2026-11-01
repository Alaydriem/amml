//! Typed access to the game for AMML mods: the shared entry ids, entry resolution with optional
//! features, and the helpers the mods share. A mod compiles this crate in; it is not a mod.

#![deny(unsafe_code)]

pub mod errors;
pub mod game;
pub mod hooks;
pub mod structs;

pub use errors::GameError;
pub use game::{
    Aabb, Actor, ActorPose, Block, BlockPos, BlockSource, Brightness, CameraLayout, CameraWriter, ChunkBlockPos,
    ChunkState, ClientInstance, Dimension, LevelChunk, Projection, RawObject, RenderCamera, SubChunk,
    ViewRenderDataLayout,
};
pub use game::{
    ActorRenderQueue, GameConnectionInfo, NetworkConnection, NetworkIdentifierEquality, NetworkSystem, OptionReader,
};
pub use game::{
    ActorRenderQueueLayout, ArrowDataLayout, ClientCacheStatusPacketLayout, FovInstructionLayout,
    GameConnectionInfoLayout, MovePlayerPacketLayout, NetworkConnectionLayout, NetworkIdentifierLayout,
    NetworkStackLatencyPacketLayout, OptionImplLayout, PlayerJoinWorldContextLayout, PrimitiveShapeLayout, TimerLayout,
};
pub use game::{
    BaseActorRenderContext, BaseActorRenderContextLayout, CaretMeasureDataLayout, CompoundTagLayout, CompoundTagRef,
    CompoundTagVariantLayout, ContainerScreenController, ItemRenderer, ItemStackBaseRef, ListTagLayout, NbtList,
    OwnedItemStack, RectangleAreaLayout, ScreenControllerRef, ScreenView, ScreenViewLayout, StringViewLayout,
    TextMeasureDataLayout, UiColorLayout, UiRenderContext, UiScene,
};
pub use game::{
    BlockLayout, BlockTypePointerLayout, BlockTypeValueLayout, DbStorageLayout, DimensionLayout, HashedStringLayout,
    LevelChunkLayout2651, LevelChunkLayout2660, LevelChunkLayoutKind, MaterialBoolsLayout, MaterialFlagsLayout,
    SubChunkLayout,
};
pub use game::{
    ContainerClosePacketLayout, ContainerOpenPacketLayout, CreativeItemEntryLayout, HitResultLayout,
    InteractPacketLayout, ItemInstanceLayout, ItemStackLayout, ItemStackNetIdVariantLayout,
    ItemStackResponseContainerInfoLayout, ItemStackResponseInfoLayout, ItemStackResponsePacketLayout,
    ItemStackResponseSlotInfoLayout, PacketLayout, ReadOnlyBinaryStreamLayout,
};
pub use structs::{
    EntryId, EntryNeed, EntryResolver, GameEntries, GameHeap, HeldClient, HookOrder, MaterialLayout, StopLatch,
};

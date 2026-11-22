//! The versioned `#[repr(C)]` contract between the AMML loader and AMML mods.
#![deny(unsafe_code)]

pub mod amml_aabb;
pub mod amml_actor_box;
pub mod amml_actor_table;
pub mod amml_actor_target;
pub mod amml_amount;
pub mod amml_api;
pub mod amml_apply_stats;
pub mod amml_band;
pub mod amml_block_entity_camera;
pub mod amml_block_entity_ghost;
pub mod amml_block_grid;
pub mod amml_block_handle;
pub mod amml_block_hit;
pub mod amml_block_info;
pub mod amml_block_match;
pub mod amml_block_pos;
pub mod amml_camera_kind;
pub mod amml_camera_table;
pub mod amml_capture_frame;
pub mod amml_capture_request;
pub mod amml_chat_field;
pub mod amml_chunk_load;
pub mod amml_client_flag;
pub mod amml_client_table;
pub mod amml_color;
pub mod amml_color_quad;
pub mod amml_column_read_stats;
pub mod amml_column_request;
pub mod amml_column_sample;
pub mod amml_connection_kind;
pub mod amml_container_action;
pub mod amml_container_entry;
pub mod amml_core_table;
pub mod amml_cursor_clip;
pub mod amml_cursor_shape;
pub mod amml_dir_kind;
pub mod amml_draw_surface;
pub mod amml_event_context;
pub mod amml_event_header;
pub mod amml_events_table;
pub mod amml_exports;
pub mod amml_face;
pub mod amml_game_type;
pub mod amml_ghost_entity_options;
pub mod amml_hand;
pub mod amml_hit_result;
pub mod amml_hold_request;
pub mod amml_image_view;
pub mod amml_inline_icon;
pub mod amml_input_blocks;
pub mod amml_input_mode;
pub mod amml_input_table;
pub mod amml_instruction_kind;
pub mod amml_inventory_move;
pub mod amml_inventory_table;
pub mod amml_item_draw;
pub mod amml_item_draw_flags;
pub mod amml_item_snapshot;
pub mod amml_join_outcome;
pub mod amml_kinds_result;
pub mod amml_lease_mode;
pub mod amml_lease_request;
pub mod amml_lease_state;
pub mod amml_light_falloff;
pub mod amml_light_source;
pub mod amml_lighting_table;
pub mod amml_log_level;
pub mod amml_lost_reason;
pub mod amml_menu;
pub mod amml_mesh_built;
pub mod amml_mesh_group;
pub mod amml_mesh_id;
pub mod amml_mesh_style;
pub mod amml_mesher_options;
pub mod amml_mod_info;
pub mod amml_modifiers;
pub mod amml_mouse_action;
pub mod amml_mouse_button;
pub mod amml_move_intent;
pub mod amml_network_table;
pub mod amml_option_key;
pub mod amml_options_table;
pub mod amml_origin;
pub mod amml_other_leases;
pub mod amml_overlay_table;
pub mod amml_permission_level;
pub mod amml_persistence;
pub mod amml_phase;
pub mod amml_player_list_entry;
pub mod amml_point;
pub mod amml_pointer_kind;
pub mod amml_pose;
pub mod amml_projection;
pub mod amml_ray_options;
pub mod amml_rect;
pub mod amml_registry_table;
pub mod amml_release_reason;
pub mod amml_render_table;
pub mod amml_request_options;
pub mod amml_request_outcome;
pub mod amml_rgba;
pub mod amml_rotation;
pub mod amml_route_via;
pub mod amml_screen;
pub mod amml_screen_kind;
pub mod amml_services;
pub mod amml_shape_op;
pub mod amml_shot_layers;
pub mod amml_size;
pub mod amml_skull_pose;
pub mod amml_slot_at;
pub mod amml_start_world_outcome;
pub mod amml_state_entry;
pub mod amml_state_value;
pub mod amml_status;
pub mod amml_str;
pub mod amml_surface_loss;
pub mod amml_surface_sample;
pub mod amml_text_align;
pub mod amml_texture_id;
pub mod amml_texture_reload;
pub mod amml_texture_source;
pub mod amml_thread;
pub mod amml_time_hold;
pub mod amml_time_state;
pub mod amml_time_table;
pub mod amml_transform;
pub mod amml_ui_profile;
pub mod amml_ui_table;
pub mod amml_use_row;
pub mod amml_uses;
pub mod amml_uv;
pub mod amml_vec3;
pub mod amml_window_change;
pub mod amml_window_table;
pub mod amml_wire_connection;
pub mod amml_wire_dir;
pub mod amml_world_info;
pub mod amml_world_table;
pub mod amml_world_text;
pub mod events;
pub mod known_log_level;
pub mod mod_id;
pub mod world_epoch;

pub use amml_aabb::AmmlAabb;
pub use amml_actor_box::AmmlActorBox;
pub use amml_actor_table::{
    ActorAabbFn, ActorBlockSourceFn, ActorBuildAtFn, ActorBuildDelayFn, ActorDimensionFn, ActorGameTypeFn,
    ActorHealthFn, ActorIsAliveFn, ActorPermissionLevelFn, ActorPickRangeFn, ActorPointFn, ActorRotationFn,
    ActorSetRotationFn, AmmlActorTable, DimensionIdFn,
};
pub use amml_actor_target::AmmlActorTarget;
pub use amml_amount::AmmlAmount;
pub use amml_api::{AMML_API_VERSION, AmmlApi, HeapDeleteFn, HeapNewFn, HookFn, LogFn, ModDirFn, ResolveFn, UnhookFn};
pub use amml_apply_stats::AmmlApplyStats;
pub use amml_band::AmmlBand;
pub use amml_block_entity_camera::AmmlBlockEntityCamera;
pub use amml_block_entity_ghost::AmmlBlockEntityGhost;
pub use amml_block_grid::AmmlBlockGrid;
pub use amml_block_handle::AmmlBlockHandle;
pub use amml_block_hit::AmmlBlockHit;
pub use amml_block_info::AmmlBlockInfo;
pub use amml_block_match::AmmlBlockMatch;
pub use amml_block_pos::AmmlBlockPos;
pub use amml_camera_kind::AmmlCameraKind;
pub use amml_camera_table::{
    AmmlCameraTable, CameraActiveFn, CameraFreeAvailableFn, CameraHideActorFn, CameraProjectionFn, CameraReleaseFreeFn,
    CameraRenderedPositionFn, CameraSetFreeFn, CameraSetPoseFn,
};
pub use amml_capture_frame::AmmlCaptureFrame;
pub use amml_capture_request::AmmlCaptureRequest;
pub use amml_chat_field::AmmlChatField;
pub use amml_chunk_load::AmmlChunkLoad;
pub use amml_client_flag::AmmlClientFlag;
pub use amml_client_table::{
    AmmlClientTable, ClientCanJoinFn, ClientCurrentFn, ClientFlagFn, ClientInGameplayFn, ClientJoinServerFn,
    ClientLeaveWorldFn, ClientLocalPlayerFn, ClientNoteActivityFn, ClientOpenMenuFn, ClientOpenPauseFn,
    ClientReplayAppFocusLostFn, ClientSelectHotbarSlotFn, ClientSendCommandFn, ClientSetFlagFn,
    ClientStartLocalWorldFn, ClientTopScreenFn, ClientWorldInfoFn, ClientWorldTextFn,
};
pub use amml_color::AmmlColor;
pub use amml_color_quad::AmmlColorQuad;
pub use amml_column_read_stats::AmmlColumnReadStats;
pub use amml_column_request::AmmlColumnRequest;
pub use amml_column_sample::AmmlColumnSample;
pub use amml_connection_kind::AmmlConnectionKind;
pub use amml_container_action::AmmlContainerAction;
pub use amml_container_entry::AmmlContainerEntry;
pub use amml_core_table::{AmmlCoreTable, AvailableFn, DirFn, GameAliveFn, LogFieldsFn};
pub use amml_cursor_clip::AmmlCursorClip;
pub use amml_cursor_shape::AmmlCursorShape;
pub use amml_dir_kind::AmmlDirKind;
pub use amml_draw_surface::AmmlDrawSurface;
pub use amml_event_context::AmmlEventContext;
pub use amml_event_header::AmmlEventHeader;
pub use amml_events_table::{AmmlEventsTable, EventCallbackFn, ListenFn, UnlistenFn};
pub use amml_exports::{AmmlExports, ModDisableFn, ModEnableFn, ModLoadFn, ModUnloadFn};
pub use amml_face::AmmlFace;
pub use amml_game_type::AmmlGameType;
pub use amml_ghost_entity_options::AmmlGhostEntityOptions;
pub use amml_hand::AmmlHand;
pub use amml_hit_result::AmmlHitResult;
pub use amml_hold_request::AmmlHoldRequest;
pub use amml_image_view::AmmlImageView;
pub use amml_inline_icon::AmmlInlineIcon;
pub use amml_input_blocks::AmmlInputBlocks;
pub use amml_input_mode::AmmlInputMode;
pub use amml_input_table::{
    AmmlInputTable, InputAcquireFn, InputButtonDownFn, InputEndLeaseFn, InputIsDownFn, InputKeyDownFn,
    InputLeaseStateFn, InputModifiersFn, InputReleaseAllFn, InputReleaseHoldFn, InputSetCoexistFn, InputSetMoveFn,
    InputSetRankingFn, InputTurnFn, InputWheelFn,
};
pub use amml_instruction_kind::AmmlInstructionKind;
pub use amml_inventory_move::AmmlInventoryMove;
pub use amml_inventory_table::{
    AmmlInventoryTable, InventoryContentsCloseFn, InventoryContentsEntryFn, InventoryContentsNameFn,
    InventoryContentsOpenFn, InventoryHeldItemFn, InventoryItemBlockFn, InventoryItemChangeStampFn,
    InventoryItemDrawReleaseFn, InventoryItemNameFn, InventoryItemReadFn, InventoryRequestMovesFn,
    InventoryRequestPickFn, InventorySelectedSlotFn, InventorySlotItemFn,
};
pub use amml_item_draw::AmmlItemDraw;
pub use amml_item_draw_flags::AmmlItemDrawFlags;
pub use amml_item_snapshot::AmmlItemSnapshot;
pub use amml_join_outcome::AmmlJoinOutcome;
pub use amml_kinds_result::AmmlKindsResult;
pub use amml_lease_mode::AmmlLeaseMode;
pub use amml_lease_request::AmmlLeaseRequest;
pub use amml_lease_state::AmmlLeaseState;
pub use amml_light_falloff::AmmlLightFalloff;
pub use amml_light_source::AmmlLightSource;
pub use amml_lighting_table::{AmmlLightingTable, LightingSetSourceFn};
pub use amml_log_level::AmmlLogLevel;
pub use amml_lost_reason::AmmlLostReason;
pub use amml_menu::AmmlMenu;
pub use amml_mesh_built::AmmlMeshBuilt;
pub use amml_mesh_group::AmmlMeshGroup;
pub use amml_mesh_id::AmmlMeshId;
pub use amml_mesh_style::AmmlMeshStyle;
pub use amml_mesher_options::AmmlMesherOptions;
pub use amml_mod_info::AmmlModInfo;
pub use amml_modifiers::AmmlModifiers;
pub use amml_mouse_action::AmmlMouseAction;
pub use amml_mouse_button::AmmlMouseButton;
pub use amml_move_intent::AmmlMoveIntent;
pub use amml_network_table::{
    AmmlNetworkTable, NETWORK_TEXT_NAME, NETWORK_TEXT_XUID, NetworkPlayerListEntryFn, NetworkPlayerListTextFn,
    NetworkPlayerSkinFn, NetworkProtocolVersionFn, NetworkSetClientChunkCacheFn, NetworkSkinAnimatedCountFn,
    NetworkSkinAnimatedFn, NetworkSkinFlagFn, NetworkSkinImageFn, NetworkSkinTextFn,
};
pub use amml_option_key::AmmlOptionKey;
pub use amml_options_table::{
    AmmlOptionsTable, OptionsGetIntFn, OptionsReleaseTransientFn, OptionsSetIntFn, OptionsViewDistanceFn,
};
pub use amml_origin::AmmlOrigin;
pub use amml_other_leases::AmmlOtherLeases;
pub use amml_overlay_table::{
    AmmlOverlayTable, OverlayCaptureFreeSlotsFn, OverlayCaptureGiveBackFn, OverlayCaptureTakeFn, OverlayCloseCaptureFn,
    OverlayCopyBackBufferFn, OverlayOpenCaptureFn, OverlayPresentCountFn, OverlayScreenshotFn,
};
pub use amml_permission_level::AmmlPermissionLevel;
pub use amml_persistence::AmmlPersistence;
pub use amml_phase::AmmlPhase;
pub use amml_player_list_entry::AmmlPlayerListEntry;
pub use amml_point::AmmlPoint;
pub use amml_pointer_kind::AmmlPointerKind;
pub use amml_pose::AmmlPose;
pub use amml_projection::AmmlProjection;
pub use amml_ray_options::AmmlRayOptions;
pub use amml_rect::AmmlRect;
pub use amml_registry_table::{AmmlRegistryTable, ProvideFn, RequireFn};
pub use amml_release_reason::AmmlReleaseReason;
pub use amml_render_table::{
    AmmlRenderTable, RenderBlockEntityCreateFn, RenderBlockEntityDrawFn, RenderBlockEntityFreeFn,
    RenderCanvasCameraPositionFn, RenderCanvasClipMatrixFn, RenderMeshBuildFn, RenderMeshDrawFn, RenderMeshFreeFn,
    RenderMesherGroupDiscardFn,
};
pub use amml_request_options::AmmlRequestOptions;
pub use amml_request_outcome::AmmlRequestOutcome;
pub use amml_rgba::AmmlRgba;
pub use amml_rotation::AmmlRotation;
pub use amml_route_via::AmmlRouteVia;
pub use amml_screen::AmmlScreen;
pub use amml_screen_kind::AmmlScreenKind;
pub use amml_services::AmmlServices;
pub use amml_shape_op::AmmlShapeOp;
pub use amml_shot_layers::AmmlShotLayers;
pub use amml_size::AmmlSize;
pub use amml_skull_pose::AmmlSkullPose;
pub use amml_slot_at::AmmlSlotAt;
pub use amml_start_world_outcome::AmmlStartWorldOutcome;
pub use amml_state_entry::AmmlStateEntry;
pub use amml_state_value::AmmlStateValue;
pub use amml_status::AmmlStatus;
pub use amml_str::AmmlStr;
pub use amml_surface_loss::AmmlSurfaceLoss;
pub use amml_surface_sample::AmmlSurfaceSample;
pub use amml_text_align::AmmlTextAlign;
pub use amml_texture_id::AmmlTextureId;
pub use amml_texture_reload::AmmlTextureReload;
pub use amml_texture_source::AmmlTextureSource;
pub use amml_thread::AmmlThread;
pub use amml_time_hold::AmmlTimeHold;
pub use amml_time_state::AmmlTimeState;
pub use amml_time_table::{AmmlTimeTable, TimeHoldFn, TimeReleaseFn, TimeStateFn, TimeStepUsFn};
pub use amml_transform::AmmlTransform;
pub use amml_ui_profile::AmmlUiProfile;
pub use amml_ui_table::{
    AmmlUiTable, UiCanDrawFn, UiChatIconsActiveFn, UiChatSetTextFn, UiChatTextFn, UiContainerActionFn,
    UiContainerCursorFn, UiContainerItemFn, UiContainerSlotCountFn, UiContainerSwapFn, UiDrawFillFn, UiDrawImageFn,
    UiDrawItemFn, UiDrawOutlineFn, UiGuiScaleFn, UiIconRegisterFn, UiIconReleaseFn, UiIconUpdateFn, UiTextureLoadFn,
    UiTextureReadyFn, UiTextureUnloadFn, UiTextureUploadRgbaFn, UiViewContainerFn, UiViewContentRectFn,
    UiViewInputModeFn, UiViewKindFn, UiViewPointerFn, UiViewSizeFn, UiViewUiProfileFn,
};
pub use amml_use_row::AmmlUseRow;
pub use amml_uses::AmmlUses;
pub use amml_uv::AmmlUv;
pub use amml_vec3::AmmlVec3;
pub use amml_window_change::AmmlWindowChange;
pub use amml_window_table::{
    AmmlWindowTable, WindowBufferSizeFn, WindowClientSizeFn, WindowClipCursorFn, WindowCursorPosFn, WindowDpiFn,
    WindowHasFocusFn, WindowIsForegroundFn, WindowReplayFocusLossFn, WindowRequestSizeFn, WindowToBufferFn,
};
pub use amml_wire_connection::AmmlWireConnection;
pub use amml_wire_dir::AmmlWireDir;
pub use amml_world_info::AmmlWorldInfo;
pub use amml_world_table::{
    AmmlWorldTable, WorldActorsInFn, WorldBiomeNameFn, WorldBlockTypeNameFn, WorldChunkFn, WorldChunkLoadStateFn,
    WorldRaycastFn, WorldReadColumnsFn, WorldScanSurfacesFn, WorldShapesApplyFn, WorldShapesMaxBatchFn,
    WorldShapesTargetEpochFn, WorldSkyDarkenFn,
};
pub use amml_world_text::AmmlWorldText;
pub use events::{
    AmmlEvent, AppFocusGained, AppFocusLost, BlockBreakContinue, BlockBreakStart, BlockChanged, BlockEntityPass,
    BuildBlock, CameraFov, CameraInstructionReceived, CameraSetup, ChatMessageAdded, ClientTick, ConnectionStarted,
    ContainerSlotHover, ContainerSlotUnhover, DimensionChanging, FrameEnd, FramePresent, FramePresented, FrameUpdate,
    GameTick, GamepadPoll, IdleScreenOpen, InjectedInputReleased, InputLeaseLost, InputLeaseResumed,
    InputLeaseSuspended, InventoryRequestDone, ItemHovertext, ItemUse, ItemUseOnBlock, JoinServerResult, JoinStarted,
    KeyInput, LatencyBarrier, LocatorBarIconDraw, LookDelta, ModsLoaded, MouseGrab, MouseInput, PacketWire,
    PlayerAttack, PlayerListUpdate, PlayerMoveInput, PlayerMoved, PlayerSkinChanged, ProviderGone, ProviderReady,
    RenderDeviceReset, ScreenChanged, ScreenClosing, ScreenRender, ScreenshotTaken, ScreenshotWritten,
    ServerJoinStarting, Shutdown, SurfaceLost, SwapChainResize, SwapChainResized, TexturesReloaded, TimeStepApplied,
    UiPointer, UiScroll, UsePressed, WindowChanged, WindowChar, WindowCursor, WindowFocus, WindowPointer, WindowSize,
    WorldJoined, WorldLeft, WorldRendered, WorldStorageClosed, WorldStorageOpened,
};
pub use known_log_level::KnownLogLevel;
pub use mod_id::ModId;
pub use world_epoch::WorldEpoch;

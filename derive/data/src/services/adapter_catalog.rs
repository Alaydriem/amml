use crate::structs::{AdapterNeed, AdapterShape, Availability, NeedKind};

static FRAME_UPDATE: [AdapterShape; 1] =
    AdapterShape::single("default", &["MinecraftGame.update", AdapterCatalog::MINECRAFT_GAME_VTABLE]);
static CLIENT_TICK: [AdapterShape; 1] = AdapterShape::single("default", &["ClientInstance.update"]);
// Version 2 raises only the ticks of the primary client: the client is the GameCallbacks subobject minus the offset.
static CLIENT_GAME_TICK: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.onTick", "ClientInstance.gameCallbacksOffset"]);
static JOIN_STARTED: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.onStartJoinGame", AdapterCatalog::CLIENT_INSTANCE_VTABLE]);
// Every server join: `MinecraftGame::joinMultiplayerWithAddress`, which the game's server joins (the Servers tab, the
// loader's `joinExternalServer`, Realms) reach, and `startExternalNetworkWorld` (the direct join). The connection info is
// passed by value: its size selects the layout of the host and port, and two members are read at derived offsets.
// Version 2 is raised for the game's joins too; version 1 was raised only through `startExternalNetworkWorld`.
static SERVER_JOIN_STARTING: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.startExternalNetworkWorld",
        AdapterCatalog::CLIENT_INSTANCE_VTABLE,
        "MinecraftGame.joinMultiplayerWithAddress",
        AdapterCatalog::MINECRAFT_GAME_VTABLE,
        "GameConnectionInfo.mUnresolvedUrl",
        "GameConnectionInfo.mExternalServerName",
        "GameConnectionInfo.size",
    ],
);
// The level's own hit result, through the local player's level: what the crosshair points at.
static PLAYER_HIT_RESULT: [AdapterShape; 1] = AdapterShape::single("default", &["Actor.mLevel", "Level.getHitResult"]);
// The direct join: the game's own `ClientInstance.connectByAddress`, which builds the connection info and the join
// context and passes them to `startExternalNetworkWorld`, called through the client subobject at the derived offset.
static CLIENT_JOIN_SERVER_DIRECT: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.connectByAddress", "ClientInstance.connectByAddressThis"]);
static WORLD_STORAGE: [AdapterShape; 1] =
    AdapterShape::single("default", &["DBStorage.ctor", "DBStorage.checkShutdownDone", "DBStorage.mFullPath"]);
// `MinecraftGame::isInGame`, a slot of the MinecraftGame vtable called on the game of the last frame.
static CLIENT_IN_GAMEPLAY: [AdapterShape; 1] = AdapterShape::single("default", &["MinecraftGame.isInGame"]);
static JOINED: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.update", "ClientInstance.getLocalPlayer"]);
// A world is left only after it was joined, so `world.left` needs the entries of `world.joined` too.
static WORLD_LEFT: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ClientExitLevel.hook", "ClientInstance.update", "ClientInstance.getLocalPlayer"],
);
static LOADER_ONLY: [AdapterShape; 1] = AdapterShape::single("loader", &[]);
// Served by the loader's window subclass. The shape is not `loader`, so nothing offers it at start: the overlay
// install offers it when its hooks are in.
static WINDOW_ONLY: [AdapterShape; 1] = AdapterShape::single(AdapterCatalog::WINDOW_SHAPE, &[]);
// Served by the loader's DXGI hooks, which need no game entry. Offered by the overlay install when its hooks are in.
static OVERLAY_ONLY: [AdapterShape; 1] = AdapterShape::single(AdapterCatalog::OVERLAY_SHAPE, &[]);

static CLIENT_READ: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ClientInstance.getLocalPlayer", "ClientInstance.isWorldActive", "ClientInstance.isShowingMenu"],
);
static CLIENT_TOP_SCREEN: [AdapterShape; 1] = AdapterShape::single("default", &["ClientInstance.getTopScreenName"]);
static CLIENT_HUD: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ClientInstance.getOptions", "IOptionRegistry.getHideHud", "IOptionRegistry.setHideGUI"],
);
static CLIENT_MOUSE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getMinecraftGame",
        "IMinecraftGame.grabMouse",
        "IMinecraftGame.releaseMouse",
        "MinecraftGame.grabMouse",
        AdapterCatalog::MINECRAFT_GAME_VTABLE,
    ],
);
static CLIENT_PAUSE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getMinecraftGame",
        "MinecraftGame.onAppFocusLost",
        "MinecraftGame.AppPlatformListener.vtable",
        "MinecraftGame.appPlatformListenerOffset",
        "ClientInstance.getOptions",
        "OptionRegistry.getIfValid",
    ],
);
// The game's handler of an app focus loss (`MinecraftGame::onAppFocusLost`), hooked by the loader.
static CLIENT_APP_FOCUS_LOST: [AdapterShape; 1] = AdapterShape::single("default", &["MinecraftGame.onAppFocusLost"]);
// The game's handler of an app focus gain: slot `onAppFocusGained` of the AppPlatformListener vtable, on the subobject
// at `appPlatformListenerOffset` of the MinecraftGame.
static CLIENT_APP_FOCUS_GAINED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "MinecraftGame.onAppFocusGained",
        "MinecraftGame.AppPlatformListener.vtable",
        "MinecraftGame.appPlatformListenerOffset",
    ],
);
// `Client::replay_app_focus_lost`: the loader calls the game's handler of the loss past its own detour.
static CLIENT_FOCUS_REPLAY: [AdapterShape; 1] = AdapterShape::single("default", &["MinecraftGame.onAppFocusLost"]);
static CLIENT_LEAVE_WORLD: [AdapterShape; 1] = AdapterShape::single("default", &["ClientInstance.requestLeaveGame"]);
static CLIENT_CAN_JOIN: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getMinecraftGame",
        "MinecraftGame.isPrimaryClientInstanceReady",
        AdapterCatalog::MINECRAFT_GAME_VTABLE,
    ],
);
static CLIENT_START_LOCAL_WORLD: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getMinecraftGame",
        AdapterCatalog::MINECRAFT_GAME_VTABLE,
        "MinecraftGame.getWorldSystem",
        "WorldSystem.mLocalWorldStarter",
        "LocalWorldStarter.startLocalWorld",
    ],
);
// The join and its result event: the result comes through the loader's callable of the same game call.
static CLIENT_JOIN_SERVER: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getMinecraftGame",
        AdapterCatalog::MINECRAFT_GAME_VTABLE,
        "MinecraftGame.getWorldSystem",
        "WorldSystem.mNetworkWorldJoiner",
        "NetworkWorldJoiner.mServerWorldJoiner",
        "ServerWorldJoiner.joinExternalServer",
    ],
);
static CLIENT_OPEN_MENU: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.getCurrentUIRouter", "OreUIRouter.pushRoute"]);
static CLIENT_SEND_COMMAND: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "MinecraftPackets.createPacket",
        "ClientInstance.getPacketSender",
        "PacketSender.send",
        "CommandRequestPacket.mCommand",
        "CommandRequestPacket.size",
    ],
);
static HUD_SELECT_HOTBAR: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "HudScreenController.vtable",
        "HudScreenController.tick",
        "HudScreenController.handleSlotSelection",
        "ScreenController.dtor",
    ],
);
static INPUT_KEY: [AdapterShape; 2] = [
    AdapterShape {
        name: "hid",
        entries: &[
            "HIDControllerGameCoreDesktop.onKeyDown",
            "HIDControllerGameCoreDesktop.onKeyUp",
            "HIDControllerGameCoreDesktop.update",
        ],
        // The game's key reset on a focus loss writes the key table without onKeyUp. Without its hook the
        // loader releases its held keys from the window procedure only.
        optional: &["HIDController.resetKeys"],
    },
    AdapterShape { name: "key_processor", entries: &["KeyboardEventProcessor.feedKeyState"], optional: &[] },
];
static INPUT_MOUSE: [AdapterShape; 2] = [
    AdapterShape { name: "seven_arg_feed", entries: &["MouseDevice.feed", "MouseDevice.instance"], optional: &[] },
    AdapterShape {
        name: "eight_arg_feed",
        entries: &["MouseDevice.feedWithControllerId", "MouseDevice.instance"],
        optional: &[],
    },
];
static INPUT_LOOK: [AdapterShape; 1] =
    AdapterShape::single("default", &["LocalPlayer.applyTurnDelta", "ClientInstance.getLocalPlayer"]);
static INPUT_MOVE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInputUpdateSystem.extractRawHIDInput",
        "RawMoveInputComponent.mInputState",
        "RawMoveInputComponent.mRawMove",
        "MoveInputState.mAnalogMoveVector",
        "MoveInputState.maskSneakTest",
        "MoveInputState.maskJumpDown",
        "MoveInputState.bitSprintDown",
        "MoveInputState.bitUp",
        "MoveInputState.bitDown",
        "MoveInputState.bitLeft",
        "MoveInputState.bitRight",
    ],
);
// The texture reload event: the entries of `ui.textures` (the loader binds the textures again), and the game function
// that applies the active resource pack stack, which the loader hooks to see a pack change.
static UI_TEXTURES_RELOADED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getTextureGroup",
        "TextureGroup.uploadTexture",
        "TextureGroup.getTexture",
        "TextureGroup.unloadTexture",
        "TextureGroup.textureGroupBase",
        "TextureGroup.baseOffset",
        "ResourceLocation.ctor",
        "MinecraftGame.applyActiveResourcePacks",
    ],
);
static INPUT_GAMEPAD: [AdapterShape; 1] = AdapterShape::single("default", &["GameControllerHandler_GameCore.refresh"]);
static CAMERA_FOV: [AdapterShape; 1] = AdapterShape::single("default", &["LevelRendererPlayer.getFov"]);
// `WorldRendered` after the game's level pass, with the graphics mode read from the option registry at each client tick.
static RENDER_WORLD_RENDERED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["LevelRenderer.renderLevel", "ClientInstance.getOptions", "IOptionRegistry.isGraphicsModeDeferred"],
);
// The world canvas: the camera position of the view of the running level renderer (the views of the frame object and
// the renderer's own camera position), and the world mesh path of `build` and `draw` (the tessellator, the mesh, the
// camera world matrix, the actor constants, the wireframe material and its four patched bytes, the name tag material
// of a loader-owned `BaseActorRenderer`). Version 3 adds the ghost draw of a block mesh: the renderer's atlas
// `TexturePtr` and the client's blend-block `MaterialPtr` (`ClientInstance.getItemInHandRenderer`,
// `ItemInHandRenderer.mMatBlendBlock`); the clip matrix reads the camera of the screen context. The mirrors of the path are checked against their entries at install. The
// optional entries are the per-call needs of `WorldCanvas::draw`: the loader's white texture goes through `ui.textures`;
// without them the area stays offered and `draw` gives `NotAvailable`.
static RENDER_WORLD_CANVAS: [AdapterShape; 1] = [AdapterShape {
    name: "default",
    entries: &[
        "FrameRenderObject.mViews",
        "ViewRenderObject.mViewData",
        "ViewRenderData.mCameraPos",
        "ViewRenderObject.size",
        "LevelRenderer.mLevelRendererPlayer",
        "LevelRendererCamera.mCameraPos",
        "ScreenContext.tessellator",
        "ScreenContext.meshContext",
        "ScreenContext.camera",
        "ScreenContext.constantBuffers",
        "ScreenContext.currentShaderColor",
        "Tessellator.begin",
        "Tessellator.end",
        "Tessellator.clear",
        "Tessellator.mTessellating",
        "Mesh.size",
        "Mesh.isValid",
        "Mesh.dtor",
        "Mesh.renderMeshTextureList",
        "MatrixStack.push",
        "MatrixStackRef.dtor",
        "Camera.mForward",
        "ActorShaderManager.setEntityConstants",
        "GlobalConstantBuffers.entityConstantBuffer",
        "LevelRendererCamera.mWireframeMaterial",
        "RenderMaterialInfo.mMaterial",
        "RenderMaterial.mEnableBlend",
        "RenderMaterial.mBlendSource",
        "RenderMaterial.mBlendDestination",
        "RenderMaterial.mDepthWriteMask",
        "BaseActorRenderer.ctor",
        "BaseActorRenderer.size",
        "LevelRenderer.mAtlasTexture",
        "ClientInstance.getItemInHandRenderer",
        "ItemInHandRenderer.mMatBlendBlock",
    ],
    optional: &UI_TEXTURES_ENTRIES,
}];
// The block mesher: a loader-owned `BlockTessellator` (game heap, constructor, `setRegion` to the block source of the
// local player, the cache window at `mBlockCache` filled from the mod's grid, `tessellateInWorld` once per render layer
// of a cell: `Block::getRenderLayer` and the `getExtraRenderLayers` slot), the shape of a cell's graphics for the back
// faces, `BlockSource::getBlock` for the cells that match the world, the registry's `minecraft:air` for the empty cells
// (`BlockTypeRegistry`, `HashedString.layout`), and the frame's tessellator to end the mesh. The
// loader offers it only while `render.world_canvas` version 3 (the draw of a block mesh) is offered. The mesh groups
// keep a game `MeshData` buffer (`MeshData.clear` builds it, `moveAssign` hands it to the frame tessellator at the
// finish, `dtor` frees it, also for a group the mod discards, on the `renderLevel` thread); the biome tint takes the tessellation policy of the block type's tint method
// (`getTessellationPolicy`, the virtual `get`).
static RENDER_BLOCK_MESHER: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockTessellator.ctor",
        "BlockTessellator.dtor",
        "BlockTessellator.setRegion",
        "BlockTessellator.tessellateInWorld",
        "BlockTessellator.mBlockCache",
        "BlockGraphics.getForBlock",
        "BlockGraphics.mBlockShape",
        "Block.getRenderLayer",
        "BlockType.getExtraRenderLayers",
        "Block.mBlockType",
        "BlockSource.getBlock",
        "Tessellator.begin",
        "Tessellator.end",
        "Tessellator.clear",
        "Tessellator.mTessellating",
        "Mesh.isValid",
        "Mesh.dtor",
        "ClientInstance.getLocalPlayer",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
        "BlockTypeRegistry.instance",
        "BlockTypeRegistry.lookupByName",
        "HashedString.layout",
        "MeshData.clear",
        "MeshData.dtor",
        "MeshData.moveAssign",
        "BiomeColorSampling.getTessellationPolicy",
        "TessellationPolicy.get",
        "BlockType.mTintMethod",
    ],
);
// Water cells of a block mesh take the biome colour: the chunk of the block source (`BlockSource::getChunk`), its biome at
// the cell (`LevelChunk::getBiome`) and the biome's water colour (`BiomeColorSampling::getWaterColor`). The loader offers
// it only with `render.block_mesher`.
static RENDER_WATER_TINT: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["BlockSource.getChunk", "LevelChunk.getBiome", "BiomeColorSampling.getWaterColor"],
);
// `BlockEntityPass`: the hook on the body every level renderer camera shares for its block entities, and the vtable of
// the local player's camera, which the loader compares to tell the player's camera from the others.
static RENDER_BLOCK_ENTITY_PASS: [AdapterShape; 1] =
    AdapterShape::single("default", &["LevelRendererCamera.renderBlockEntitiesBody", "LevelRendererPlayer.vtable"]);
// Block entity ghosts: the pass, the block actor of a block (the factory, or `loadStatic` with NBT read through
// `NbtIo::read` over a `StringByteInput` and a `DefaultDataLoadHelper`, on the level of the local player), and its draw
// through the dispatcher of the pass context (`_getRenderComponent`, `BlockActorRenderDispatcher::render` with a
// disengaged `optional<RenderMetadata>` and the block source of the local player); the actors are released before the
// game's `renderLevel`. 26.63 has the factory without a flag;
// 70.x gives it the byte of the level's flag slot, as `loadStatic` does.
static RENDER_BLOCK_ENTITIES: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::BLOCK_ENTITIES_FACTORY_SHAPE,
        entries: &[
            "LevelRendererCamera.renderBlockEntitiesBody",
            "LevelRendererPlayer.vtable",
            "BlockActor.loadStatic",
            "BlockActor._getRenderComponent",
            "DefaultDataLoadHelper.vtable",
            "BlockActorRenderDispatcher.render",
            "RenderMetadata.engaged",
            "BaseActorRenderContext.mBlockEntityRenderDispatcher",
            "BaseActorRenderContext.mImpl",
            "BaseActorRenderContext.Impl.mCameraTargetPosition",
            "NbtIo.read",
            "StringByteInput.vtable",
            "CompoundTag.dtor",
            "Actor.mLevel",
            "ClientInstance.getLocalPlayer",
            "Actor.getDimension",
            "Dimension.getBlockSourceFromMainChunkSource",
            "Block.mBlockType",
            "LevelRenderer.renderLevel",
            "VanillaBlockActorFactory.createBlockActor",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::BLOCK_ENTITIES_FLAG_SHAPE,
        entries: &[
            "LevelRendererCamera.renderBlockEntitiesBody",
            "LevelRendererPlayer.vtable",
            "BlockActor.loadStatic",
            "BlockActor._getRenderComponent",
            "DefaultDataLoadHelper.vtable",
            "BlockActorRenderDispatcher.render",
            "RenderMetadata.engaged",
            "BaseActorRenderContext.mBlockEntityRenderDispatcher",
            "BaseActorRenderContext.mImpl",
            "BaseActorRenderContext.Impl.mCameraTargetPosition",
            "NbtIo.read",
            "StringByteInput.vtable",
            "CompoundTag.dtor",
            "Actor.mLevel",
            "ClientInstance.getLocalPlayer",
            "Actor.getDimension",
            "Dimension.getBlockSourceFromMainChunkSource",
            "Block.mBlockType",
            "LevelRenderer.renderLevel",
            "VanillaBlockActorFactory.createBlockActorWithFlag",
            "Level.isClientSide",
        ],
        optional: &[],
    },
];
// The lead of a large chest ghost: the game's chest type test, the actor's position and the chest's pair members.
static RENDER_CHEST_PAIR: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockActor.isType",
        "BlockActor.mPosition",
        "ChestBlockActor.mPairFlags",
        "ChestBlockActor.mLargeChestPaired",
        "ChestBlockActor.mLargeChestPairedPosition",
    ],
);
// A skull ghost drawn by the dispatcher's skull renderer with the name of the ghost's block type (a `HashedString` of a
// known layout) after the game's own skull type test. 26.63 has `renderSkull` with the face and the turn; 70.x has only
// `renderSkullUpright`, so the loader pushes the camera's world matrix stack (`ScreenContext.camera`, the camera mirror
// anchored by `Camera.mForward`) and multiplies the face and turn transform first.
static RENDER_SKULL_POSE: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::SKULL_RENDER_SHAPE,
        entries: &[
            "SkullBlockRenderer.renderSkull",
            "BlockActorRenderDispatcher.mSkullRenderer",
            "BlockType.mNameInfo.mFullName",
            "HashedString.layout",
            "BlockActor.isType",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::SKULL_UPRIGHT_SHAPE,
        entries: &[
            "SkullBlockRenderer.renderSkullUpright",
            "BlockActorRenderDispatcher.mSkullRenderer",
            "MatrixStack.push",
            "MatrixStackRef.dtor",
            "BlockType.mNameInfo.mFullName",
            "HashedString.layout",
            "BlockActor.isType",
            "ScreenContext.camera",
            "Camera.mForward",
        ],
        optional: &[],
    },
];
// `CameraSetup` Before and After the game's camera setup of a rendered frame.
static CAMERA_SETUP: [AdapterShape; 1] = AdapterShape::single("default", &["LevelRendererPlayer.setupCamera"]);
// `Camera::set_pose`: the rotation written into the `mce::Camera` after `setupCamera` (mirror anchored by
// `Camera.mForward`, then `updateViewMatrixDependencies`), the position written into the view object after
// `createViewRenderObject` and into the level camera after `preRenderUpdate`. The mirrors are checked against their
// entries at install. The `setupCamera` hook of `camera.setup` is needed as well.
static CAMERA_POSE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LevelRendererPlayer.createViewRenderObject",
        "LevelRendererCamera.preRenderUpdate",
        "LevelRendererCamera.mCameraPos",
        "LevelRendererCamera.mCameraTargetPos",
        "Camera.mForward",
        "Camera.updateViewMatrixDependencies",
        "ViewRenderObject.mViewData",
        "ViewRenderData.mCameraPos",
    ],
);
// `Camera::projection`: the top of the camera's projection stack, through the `mce::Camera` mirror anchored by
// `Camera.mForward`, read in `CameraSetup` After.
static CAMERA_PROJECTION: [AdapterShape; 1] = AdapterShape::single("default", &["Camera.mForward"]);
// `CameraControl::{set_free, release_free, free_available}`: the loader captures the client network handler from its
// hooked handler slots (the camera instruction and chunk publisher slots of the table it patches), makes a
// `CameraInstructionPacket` with the game's factory, writes the set or clear instruction through the
// `CameraInstruction` mirror (checked against the packet size and the spline offset) with the index of the
// `minecraft:free` preset of the local player's level, and hands it to the captured handler's camera slot. The captured
// handler is forgotten at the world leave and at the next join start, so the row needs those session hooks too.
static CAMERA_FREE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.CameraInstructionPacket",
        "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket",
        "MinecraftPackets.createPacket",
        "CameraInstructionPacket.size",
        "CameraInstructionPacket.mCameraInstruction",
        "CameraInstruction.mSpline",
        "Level.getCameraPresets",
        "CameraPreset.size",
        "Actor.mLevel",
        "ClientExitLevel.hook",
        "ClientInstance.onStartJoinGame",
        AdapterCatalog::CLIENT_INSTANCE_VTABLE,
    ],
);
// `WorldShapes::{apply, max_batch, target_epoch}`: the client network handler captured as for `camera.free` (its
// hooked slots and the session hooks that forget it), a `PrimitiveShapesPacket` from the game's factory whose shape
// vector (`PrimitiveShapesPacket.mShapes`) the loader fills with payloads (checked against
// `PrimitiveShapeDataPayload.size`) in a block of the game heap, handed to the captured handler's shapes slot.
static WORLD_SHAPES: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.CameraInstructionPacket",
        "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket",
        "ClientNetworkHandler.handle.PrimitiveShapesPacket",
        "MinecraftPackets.createPacket",
        "PrimitiveShapesPacket.mShapes",
        "PrimitiveShapeDataPayload.size",
        "core.operator_new",
        "core.operator_delete",
        "ClientExitLevel.hook",
        "ClientInstance.onStartJoinGame",
        AdapterCatalog::CLIENT_INSTANCE_VTABLE,
    ],
);
// `CameraControl::active_camera`: the camera registry of the client (`getCameraRegistry`), its camera entity vector and
// the game's entt lookup of the `CameraComponent` (its name) and the `ActiveCameraComponent` of each camera entity. The
// name is a `HashedString` of the layout `HashedString.layout` names. The component size is the stride of its payload
// pages: 26.63 builds it from a factor and a shift, 70.x has it as one value.
static CAMERA_ACTIVE: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::ACTIVE_CAMERA_FACTOR_SHAPE,
        entries: &[
            "ClientInstance.getCameraRegistry",
            "CameraRegistry.mCameraEntities",
            "CameraRegistry.cameraEntityStride",
            "CameraComponent.typeId",
            "ActiveCameraComponent.typeId",
            "HashedString.layout",
            "CameraComponent.sizeFactor",
            "CameraComponent.sizeShift",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::ACTIVE_CAMERA_SIZE_SHAPE,
        entries: &[
            "ClientInstance.getCameraRegistry",
            "CameraRegistry.mCameraEntities",
            "CameraRegistry.cameraEntityStride",
            "CameraComponent.typeId",
            "ActiveCameraComponent.typeId",
            "HashedString.layout",
            "CameraComponent.size",
        ],
        optional: &[],
    },
];
// `camera.hide_actor`: the local player is erased from the actor render queue after `queueRenderEntities` and its
// dispatcher draw is skipped. The queue is the member of `LevelRendererCamera` the erase works on.
static CAMERA_HIDE_ACTOR: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LevelRendererPlayer.vtable",
        "LevelRendererPlayer.queueRenderEntities",
        "LevelRendererCamera.mActorRenderQueue",
        "ActorRenderDispatcher.render",
        "ClientInstance.getLocalPlayer",
    ],
);
// `PlayerMoved` After the game's handler of a `MovePlayerPacket` whose runtime id is the local player's.
static PLAYER_MOVED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.MovePlayerPacket",
        "Actor.getRuntimeID",
        "ClientInstance.getLocalPlayer",
    ],
);
// `CameraInstructionReceived` Before the game's handler of a `CameraInstructionPacket`; its set and clear parts are read
// through the `CameraInstruction` mirror (checked against the packet size and the spline offset).
static CAMERA_INSTRUCTION_RECEIVED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.CameraInstructionPacket",
        "CameraInstructionPacket.mCameraInstruction",
        "CameraInstructionPacket.size",
        "CameraInstruction.mSpline",
    ],
);
static DIMENSION_CHANGING: [AdapterShape; 1] =
    AdapterShape::single("default", &["LevelRendererPlayer.onWillChangeDimension"]);
// The game's any-input handler: getMinecraftInput()->mInputHandler->mAnyInputHandler.
static CLIENT_ACTIVITY: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ClientInstance.getMinecraftInput", "MinecraftInputHandler.mInputHandler", "InputHandler.mAnyInputHandler"],
);
// `Options::view_distance_chunks`: the client's option registry and its `getViewDistanceChunks` slot.
static OPTIONS_VIEW_DISTANCE: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.getOptions", "IOptionRegistry.getViewDistanceChunks"]);
// `Options::{get_int, set_int, release_transient}`: the option registry, an option by id (the scan for the save tag of
// vsync), the effective max frame rate option and the int option's setter.
static OPTIONS_INT: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getOptions",
        "OptionRegistry.getIfValid",
        "OptionRegistry.getEffectiveMaxFramerateOption",
        "IntOption.set",
    ],
);
static IDLE_SCREEN_OPEN: [AdapterShape; 1] =
    AdapterShape::single("default", &["OreUIRouter.pushRoute", "OreUIRouter.replaceRoute"]);

// The local player of the primary client, read through its Actor vtable. Later appends to the actor table raise the
// row's version (`AdapterNeed::area_at`).
// Version 2 adds the position, eye position, rotation and dimension reads; version 3 the head position and the block
// source of the dimension; version 4 the view vector (the box reads the shape component version 2 already names).
static ACTOR_READ: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "Actor.isAlive",
        "ClientInstance.getLocalPlayer",
        "Actor.stateVectorComponent",
        "StateVectorComponent.mPos",
        "Actor.aabbShapeComponent",
        "Actor.actorRotationComponent",
        "ActorRotationComponent.mRot",
        "Actor.getDimension",
        "Dimension.getDimensionId",
        "Actor.getAttachPos",
        "Dimension.getBlockSourceFromMainChunkSource",
        "Actor.getViewVector",
    ],
);
// The game mode of the local player: `Player::getPlayerGameType`.
static ACTOR_GAME_TYPE: [AdapterShape; 1] = AdapterShape::single("default", &["Player.getPlayerGameType"]);
// The player permission level of the local player: 26.63 has the getter, 70.x reads the byte of the abilities
// component the getter `Player.getAbilitiesComponent` gives. The shape holds the entries that resolve on the build.
static ACTOR_PERMISSION: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::PERMISSION_GETTER_SHAPE,
        entries: &["Player.getPlayerPermissionLevel"],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::PERMISSION_ABILITIES_SHAPE,
        entries: &["Player.getAbilitiesComponent", "AbilitiesComponent.mPlayerPermissions"],
        optional: &[],
    },
];
// The rotation of the local player written in place: the current and previous rotation of the rotation component,
// and the head yaw call.
static ACTOR_LOOK_WRITE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["Actor.actorRotationComponent", "ActorRotationComponent.mRot", "Actor.setYHeadRotations"],
);
// The health of the local player: the attribute read of the entity context the actor holds.
static ACTOR_HEALTH: [AdapterShape; 1] =
    AdapterShape::single("default", &["ActorAttribute.getHealth", "Actor.mEntityContext"]);
// The first block a ray meets in the block source of the local player: `BlockSource::clip` through its vtable slot,
// with a `std::function` of the loader. The block source comes from the actor chain.
static WORLD_RAYCAST: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["BlockSource.clip", "Actor.getDimension", "Dimension.getBlockSourceFromMainChunkSource"],
);
// The boxes of the actors in a box of the block source of the local player: `BlockSource::fetchEntities` through its
// vtable slot, the word order of the span it gives (`BlockSource.fetchEntitiesSpanOrder`), the box of each actor
// through its shape component, and the local player to mark it.
static WORLD_ACTORS_IN: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockSource.fetchEntities",
        "BlockSource.fetchEntitiesSpanOrder",
        "Actor.aabbShapeComponent",
        "ClientInstance.getLocalPlayer",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
    ],
);
// The block states of the game, as owned block handles: the registry lookup by name (the registry is
// `BlockTypeRegistry.instance` + 18h, the address both instance recipes subtract 18h from) for the type, its default
// state and the `version` of that state's serialization tag (`Block.mSerializationId`, read through the `CompoundTag`
// map mirror); the block tag the loader writes, read by `NbtIo::read` over a loader-made `StringByteInput` and looked up
// by `tryGetBlockFromNBT`, then destroyed; the five transform utils for a turn; the name of the block type; the item of
// a block (`asItemInstance`, read through the item stack mirror of `inventory.read`, then destroyed).
static WORLD_BLOCK_STATES: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockTypeRegistry.instance",
        "BlockTypeRegistry.lookupByName",
        "Block.mSerializationId",
        "Block.mBlockType",
        "BlockType.mNameInfo.mFullName",
        "HashedString.layout",
        "CompoundTag.nodeVariantIndex",
        "CompoundTag.vtable",
        "StringByteInput.vtable",
        "NbtIo.read",
        "BlockSerializationUtils.tryGetBlockFromNBT",
        "CompoundTag.dtor",
        "VanillaBlockStateTransformUtils.transformBlock",
        "VanillaBlockStateTransformUtils.transformConnectionStates",
        "VanillaBlockStateTransformUtils.transformWallConnectionTypes",
        "VanillaBlockStateTransformUtils.hasTransformableConnectionStates",
        "VanillaBlockStateTransformUtils.hasTransformableWallStates",
        "BlockType.asItemInstance",
        "ItemStackBase.dtor",
    ],
);
// The block at a position of the block source of the local player (`BlockSource::getBlock`), the type of the block
// (`Block.mBlockType`, its name and material as the chunk reads do, so the ids are the ones of `world.chunk_read`), and
// the type's `canBeBuiltOver`, `isInteractiveBlock` and `asItemInstance` slots; the item stack is destroyed after the
// read (`ItemStackBase.dtor`). The block source comes from the actor chain.
static WORLD_BLOCKS: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockSource.getBlock",
        "Block.mBlockType",
        "BlockType.canBeBuiltOver",
        "BlockType.isInteractiveBlock",
        "BlockType.asItemInstance",
        "BlockType.mNameInfo.mFullName",
        "BlockType.mMaterialInline",
        "ItemStackBase.dtor",
        "HashedString.layout",
        "ClientInstance.getLocalPlayer",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
    ],
);
// The distinct block types of a box of the block source of the local player: the chunk and its load state to know the box
// is loaded (`BlockSource::getChunk`, the chunk layout), the block of each cell (`BlockSource::getBlock`) and its type id
// as the chunk reads give it, so the ids are the ones of `world.chunk_read`.
static WORLD_KINDS_IN_BOX: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockSource.getChunk",
        "BlockSource.getBlock",
        "LevelChunk.mSubChunks",
        "LevelChunk.getBlock",
        "SubChunk.size",
        "SubChunk.isPlaceHolderSubChunk",
        "Block.mBlockType",
        "BlockType.mNameInfo.mFullName",
        "BlockType.mMaterialInline",
        "HashedString.layout",
        "ClientInstance.getLocalPlayer",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
    ],
);
// The sky darkness of the dimension of the local player.
static WORLD_SKY_DARKEN: [AdapterShape; 1] =
    AdapterShape::single("default", &["Dimension.mSkyDarken", "Actor.getDimension"]);
// The chunk of the block source of the local player, how much of it the client has, the walk of its columns and the
// names of block types. The block source comes from the actor chain.
static WORLD_CHUNK_READ: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockSource.getChunk",
        "LevelChunk.mSubChunks",
        "LevelChunk.getBlock",
        "SubChunk.size",
        "SubChunk.isPlaceHolderSubChunk",
        "BlockType.mMaterialInline",
        "BlockType.mNameInfo.mFullName",
        "HashedString.layout",
        "Block.mBlockType",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
    ],
);
// The game's own map colours in the column read: 26.63 reads the map colour component of the block and tints it by
// its biome policy (the block type's own colour when the block has no component); 70.x asks the block. Both read the
// water and grass colours of the biome the chunk holds, through the main chunk source of the chunk's dimension.
static WORLD_MAP_COLOR: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::MAP_COLOR_COMPONENT_SHAPE,
        entries: &[
            "Block.tryGetComponent.BlockMapColorComponent",
            "BiomeColorSampling.getMapPolicy",
            "MapPolicy.get",
            "BlockType.getMapColor",
            "Block.mBlockType",
            "Dimension.getBlockSourceFromMainChunkSource",
            "BiomeColorSampling.getWaterColor",
            "BiomeColorSampling.getMapGrassColor",
            "LevelChunk.getBiome",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::MAP_COLOR_BLOCK_SHAPE,
        entries: &[
            "Block.getMapColor",
            "Block.mBlockType",
            "Dimension.getBlockSourceFromMainChunkSource",
            "BiomeColorSampling.getWaterColor",
            "BiomeColorSampling.getMapGrassColor",
            "LevelChunk.getBiome",
        ],
        optional: &[],
    },
];
// A block change of the block source of the local player: the loader's own listener object (a copy of the base listener
// vtable with the two slots replaced), attached to and detached from the block source, the dimension id of the source,
// and the block type ids of the chunk reads. The block source comes from the actor chain.
static BLOCK_CHANGED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockSourceListener.vtable",
        "BlockSourceListener.onSourceDestroyed",
        "BlockSourceListener.onBlockChanged",
        "BlockSource.addListener",
        "BlockSource.removeListener",
        "BlockSource.getDimensionId",
        "Block.mBlockType",
        "BlockType.mMaterialInline",
        "BlockType.mNameInfo.mFullName",
        "HashedString.layout",
        "ClientInstance.getLocalPlayer",
        "Actor.getDimension",
        "Dimension.getBlockSourceFromMainChunkSource",
    ],
);
// The facts of the joined world: the connection info of the client (the getter, its size and its destructor), the level
// of the client with its seed and level data, the realm flag, and the storage the game opened for the world (the two
// hooked functions keep the live storage; its folder is a derived member).
static CLIENT_WORLD_INFO: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "ClientInstance.getGameConnectionInfo",
        "ClientInstance.getLevel",
        "ClientInstance.isInRealm",
        "GameConnectionInfo.size",
        "GameConnectionInfo.dtor",
        "Level.getLevelSeed64",
        "ClientLevel.getLevelData",
        "LevelData.mLevelName",
        "LevelData.mCommandsEnabled",
        "DBStorage.vtable",
        "DBStorage.ctor",
        "DBStorage.checkShutdownDone",
        "DBStorage.mFullPath",
    ],
);
// The biome id of a column and the names behind the ids.
static WORLD_BIOMES: [AdapterShape; 1] =
    AdapterShape::single("default", &["LevelChunk.getBiome", "HashedString.layout", "Biome.mId", "Biome.mHash"]);
// The surfaces of a column: the chunk reads, the support of a block (26.63 takes the face, 70.x the face and a flag), the
// collision shape of a block type and the light of a block position.
static WORLD_SCAN_SURFACES: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::SURFACE_SUPPORT_SHAPE,
        entries: &[
            "BlockSource.getChunk",
            "LevelChunk.mSubChunks",
            "LevelChunk.getBlock",
            "SubChunk.size",
            "SubChunk.isPlaceHolderSubChunk",
            "BlockType.mMaterialInline",
            "BlockType.mNameInfo.mFullName",
            "HashedString.layout",
            "Block.mBlockType",
            "Actor.getDimension",
            "Dimension.getBlockSourceFromMainChunkSource",
            "Block.canProvideFullSupport",
            "BlockType.getCollisionShape",
            "BlockSource.getBrightnessPair",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::SURFACE_SUPPORT_FLAG_SHAPE,
        entries: &[
            "BlockSource.getChunk",
            "LevelChunk.mSubChunks",
            "LevelChunk.getBlock",
            "SubChunk.size",
            "SubChunk.isPlaceHolderSubChunk",
            "BlockType.mMaterialInline",
            "BlockType.mNameInfo.mFullName",
            "HashedString.layout",
            "Block.mBlockType",
            "Actor.getDimension",
            "Dimension.getBlockSourceFromMainChunkSource",
            "Block.canProvideFullSupportWithFlag",
            "BlockType.getCollisionShape",
            "BlockSource.getBrightnessPair",
        ],
        optional: &[],
    },
];

// The held stack and the selected slot of the local player, read through the inventory chain and the ItemStack mirror;
// the names through the HashedString mirror that `HashedString.layout` selects.
static INVENTORY_READ: [AdapterShape; 1] = [AdapterShape {
    name: "default",
    entries: &[
        "ClientInstance.getLocalPlayer",
        "Player.playerInventory",
        "PlayerInventory.mInventory",
        "PlayerInventory.mSelected",
        "FillingContainer.mItems",
        "ItemStack.size",
        "ItemStackBase.mItem",
        "ItemStackBase.mCount",
        "ItemStackBase.getDamageValue",
        "Item.getMaxDamage",
        "Item.mFullName",
        "HashedString.layout",
        // Version 2: the off hand, and the block a stack places with its light emission.
        "Actor.getItemInHandSlot",
        "ItemStackBase.mBlock",
        "Block.mDirectData.mLightEmission",
        // Version 4: the contents of a shulker box or a bundle.
        "ItemStack.fromTag",
        "ItemStackBase.dtor",
        "CompoundTag.nodeVariantIndex",
        "CompoundTag.vtable",
    ],
    // Version 3: each optional entry switches on one field of the item snapshot (max count, glint, the stack
    // check of a slot, the tag writer, which no call uses yet); a build without one serves the
    // rest.
    // Version 4 needs the entries of the contents (a game stack made from each tag, whose user data the loader walks
    // with layout mirrors) and, with them, the tag map anchor and the `CompoundTag` vtable that names a compound element.
    optional: &["ItemStackBase.getMaxStackSize", "Item.isGlint", "ItemStack.vtable", "CompoundTag.write"],
}];
// The loader-owned held light: the terrain light getter, the rebuild through the render chunk coordinators in
// renderLevel, and the Vibrant Visuals static light pass (skipped near the light without an emitter).
static LIGHTING_TERRAIN: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BlockTessellatorCache.getLightColor",
        "LevelRenderer.renderLevel",
        "LevelRenderer.mRenderChunkCoordinators",
        "RenderChunkCoordinator._setDirty",
        "RenderChunkCoordinator.vtable",
        "LightPropagationCoordinator.propagateForMesh",
    ],
);
static LIGHTING_ENTITY: [AdapterShape; 1] = AdapterShape::single("default", &["BlockSource.getLightColor"]);
// The hand emitter: the grid fill of the static light spread (one entry per signature), the game's TLS index and the
// emitter vector in its TLS block, and the game heap the vector grows on.
static LIGHTING_VIBRANT: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::LIGHT_GRID_SHAPE,
        entries: &[
            "LightPropagation.fillSubChunkGrid",
            "core.tls_index",
            "LightPropagation.tlsEmitterVector",
            "core.operator_new",
            "core.operator_delete",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::LIGHT_GRID_NO_MODE_SHAPE,
        entries: &[
            "LightPropagation.fillSubChunkGridNoMode",
            "core.tls_index",
            "LightPropagation.tlsEmitterVector",
            "core.operator_new",
            "core.operator_delete",
        ],
        optional: &[],
    },
];
// The player-action events: loader hooks on the GameMode slots (the base table, and for the attack the survival table
// too) and on Item::useOn; the owner is GameMode::mPlayer or the useOn actor, compared with the local player.
static BLOCK_BREAK_START: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["GameMode.vtable", "GameMode.startDestroyBlock", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"],
);
static BLOCK_BREAK_CONTINUE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["GameMode.vtable", "GameMode.continueDestroyBlock", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"],
);
// One shape per useItem signature (`GameMode.useItemResult`): a bool result, or an InteractionResult through a hidden
// pointer; each slot entry resolves only on its own builds.
static ITEM_USE: [AdapterShape; 2] = [
    AdapterShape {
        name: "bool_result",
        entries: &["GameMode.vtable", "GameMode.useItem", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"],
        optional: &[],
    },
    AdapterShape {
        name: "interaction_result",
        entries: &[
            "GameMode.vtable",
            "GameMode.useItemInteraction",
            "GameMode.mPlayer",
            "ClientInstance.getLocalPlayer",
        ],
        optional: &[],
    },
];
static PLAYER_ATTACK: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["GameMode.vtable", "SurvivalMode.vtable", "GameMode.attack", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"],
);
static ITEM_USE_ON_BLOCK: [AdapterShape; 1] =
    AdapterShape::single("default", &["Item.useOn", "ClientInstance.getLocalPlayer"]);
// The use press: a loader hook on the static `ClientInputCallbacks::handleBuildAction`.
static USE_PRESSED: [AdapterShape; 1] = AdapterShape::single("default", &["ClientInputCallbacks.handleBuildAction"]);
// The build: a loader hook on the base body in slot 6 of the GameMode table, which every game mode's build reaches; the
// owner is GameMode::mPlayer, compared with the local player.
static BUILD_BLOCK: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["GameMode.vtable", "GameMode.buildBlock", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"],
);
// A mod's own build: the game mode of the player, the level's hit result that the build reads, the build slot, and the
// build repeat delay of the game mode.
static PLAYER_BUILD: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["Player.mGameMode", "Actor.mLevel", "Level.getHitResult", "GameMode.buildBlock", "GameMode.buildingTickDelay"],
);
// The pick range: 26.63 has GameMode::getMaxPickRange; 70.x inlines it, so the loader calls Player::isCreative and
// takes one of the two constants the game loads.
static PLAYER_PICK_RANGE: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::PICK_RANGE_GETTER_SHAPE,
        entries: &["Player.mGameMode", "GameMode.getMaxPickRange"],
        optional: &[],
    },
    AdapterShape { name: AdapterCatalog::PICK_RANGE_CREATIVE_SHAPE, entries: &["Player.isCreative"], optional: &[] },
];

// The screen views: the detour on the view draw, and the render context the draw writes to.
static UI_SCREEN_RENDER: [AdapterShape; 1] =
    AdapterShape::single("default", &["ScreenView.render", "MinecraftUIRenderContext.mScreenContext"]);
// The reads of one screen view: its controller and size, the pointer and input mode, the visual tree walk to the
// content panel (names, children, cached position and size, visibility), the UI profile option of the client, and
// the container test of the controller.
static UI_SCREEN: [AdapterShape; 1] = [AdapterShape {
    name: "default",
    entries: &[
        "ScreenView.mController",
        "ScreenView.mSize",
        "ScreenView.mVisualTree",
        "ScreenView.mPointerLocationPrevious",
        "ScreenView.mInputMode",
        "VisualTree.mRootControl",
        "UIControl.mName",
        "UIControl.mChildren",
        "UIControl.mCachedPosition",
        "UIControl.mCachedPositionDirty",
        "UIControl.mCachedPositionDirtyMask",
        "UIControl.mIsVisibleInTree",
        "UIControl.mIsVisibleInTreeMask",
        "UIControl.mSize",
        "UIControl._setCachedPosition",
        "ClientInstance.getOptions",
        "IOptionRegistry.getUIProfile",
        "ScreenController.isContainerScreen",
        // Version 2: the container calls (the visual stack read, the four move handlers, the container manager
        // controller and its swap).
        "ContainerScreenController.getVisualItemStackImpl",
        "ContainerScreenController._handleAutoPlace",
        "ContainerScreenController._handleTakeAll",
        "ContainerScreenController._handlePlaceAll",
        "ContainerScreenController._handleDropItem",
        "ContainerScreenController._handleDropSelectedItem",
        "ContainerScreenController.mContainerManagerController",
        "ContainerManagerController.handleSwap",
    ],
    optional: &[],
}];
// The hover handlers of a container screen; the controller of a hover gets its screen id from the view draw.
static UI_CONTAINER_SLOT_HOVER: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ContainerScreenController.onContainerSlotHovered", "ScreenView.render", "ScreenView.mController"],
);
static UI_CONTAINER_SLOT_UNHOVER: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ContainerScreenController.onContainerSlotUnhovered", "ScreenView.render", "ScreenView.mController"],
);
// The button handler of a screen view and the members it tests: the pointer button ids and the pointer position. The
// input mode adds to the event and is not needed for it.
static INPUT_UI_POINTER: [AdapterShape; 1] = [AdapterShape {
    name: "default",
    entries: &["ScreenView.handleButtonEvent", "ScreenView.mPointerButtonIds", "ScreenView.mPointerLocationPrevious"],
    optional: &["ScreenView.mInputMode"],
}];
// The direction handler of a scene (a wheel notch is its Scroll direction), the view of the scene, and the vtable that
// proves the handler is a scene virtual.
static INPUT_UI_SCROLL: [AdapterShape; 1] =
    AdapterShape::single("default", &["UIScene.handleDirection", "UIScene.mScreenView", "UIScene.vtable"]);
// The shulker box override of the hover text and the base text it falls back to.
static UI_ITEM_HOVERTEXT: [AdapterShape; 1] =
    AdapterShape::single("default", &["ShulkerBoxBlockItem.appendFormattedHovertext", "Item.appendFormattedHovertext"]);
// Moves between the slots of the player inventory as item stack requests: the packet factory, the client's packet
// sender and its send, the read of a packet from payload bytes through a read-only stream, the runtime id of the local
// player for the `Interact` packet, the slot swap of the player container and the net id of a stack, the inventory chain
// to the slots, and the three handlers of the client's network handler the loader watches (the server's open and close of
// the hidden inventory and the answers).
static INVENTORY_REQUEST: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "MinecraftPackets.createPacket",
        "ClientInstance.getPacketSender",
        "PacketSender.send",
        "Packet.read",
        "ReadOnlyBinaryStream.vtable",
        "ReadOnlyBinaryStream.dtor",
        "Actor.getRuntimeID",
        "FillingContainer.swapSlots",
        "ItemStack.mNetIdVariant",
        "ClientInstance.getLocalPlayer",
        "Player.playerInventory",
        "PlayerInventory.mInventory",
        "FillingContainer.mItems",
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.ContainerOpenPacket",
        "ClientNetworkHandler.handle.ContainerClosePacket",
        "ClientNetworkHandler.handle.ItemStackResponsePacket",
    ],
);
// The creative pick, version 2 of `inventory.request`: everything the moves need, then the creative item list of the
// player's level (the level from the player, its item registry, the creative registry and its entries), the item of a
// block (`asItemInstance`), the stack constructors and `reinit`, the slot write (`setItem`) that shows the prediction, the
// stack limit and the game mode, which must be creative.
static INVENTORY_REQUEST_PICK: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "MinecraftPackets.createPacket",
        "ClientInstance.getPacketSender",
        "PacketSender.send",
        "Packet.read",
        "ReadOnlyBinaryStream.vtable",
        "ReadOnlyBinaryStream.dtor",
        "Actor.getRuntimeID",
        "FillingContainer.swapSlots",
        "ItemStack.mNetIdVariant",
        "ClientInstance.getLocalPlayer",
        "Player.playerInventory",
        "PlayerInventory.mInventory",
        "FillingContainer.mItems",
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.ContainerOpenPacket",
        "ClientNetworkHandler.handle.ContainerClosePacket",
        "ClientNetworkHandler.handle.ItemStackResponsePacket",
        "Actor.mLevel",
        "Level.getItemRegistry",
        "ItemRegistryRef.getCreativeItemRegistry",
        "CreativeItemRegistry.mCreativeItems",
        "ItemStack.emptyCtor",
        "ItemStack.copyCtor",
        "ItemStack.reinit",
        "ItemStack.vtable",
        "ItemStackBase.getMaxStackSize",
        "BlockType.asItemInstance",
        "Player.getPlayerGameType",
        "FillingContainer.setItem",
    ],
);
// The destructor that every screen controller reaches; the HUD adapter hooks the same function.
static UI_SCREEN_CLOSING: [AdapterShape; 1] = AdapterShape::single("default", &["ScreenController.dtor"]);
// The top screen is the controller of the last view drawn; the name tells a toast from a screen.
static UI_SCREEN_CHANGED: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.getTopScreenName", "ClientInstance.update", "ScreenView.render"]);

// The draw calls of the render context: the three draw virtuals and the image flush, the screen context and GUI
// data that give the GUI scale, and the texture request. Its optional entry is the queue of images the game has not
// drawn yet.
static UI_DRAW: [AdapterShape; 1] = [AdapterShape {
    name: "default",
    entries: &[
        "MinecraftUIRenderContext.fillRectangle",
        "MinecraftUIRenderContext.drawRectangle",
        "MinecraftUIRenderContext.drawImage",
        "MinecraftUIRenderContext.flushImages",
        "MinecraftUIRenderContext.getTexture",
        "MinecraftUIRenderContext.vtable",
        "MinecraftUIRenderContext.mScreenContext",
        "ScreenContext.guiData",
        "GuiData.getGuiScale",
        // Version 2: the item draw (`UiDraw::item`): the item renderer call, the render context made around it, the
        // client's game and the item's animation frame.
        "ItemRenderer.renderGuiItemNew",
        "BaseActorRenderContext.ctor",
        "BaseActorRenderContext.dtor",
        "BaseActorRenderContext.size",
        "ClientInstance.getMinecraftGame",
        "Item.getAnimationFrameFor",
    ],
    optional: &["MinecraftUIRenderContext.mQueuedImages"],
}];
// The texture requests: the render context asks the game for a texture by its resource location; an upload goes
// through the texture group of the client (the image buffer is filled in the loader, the group takes it).
static UI_TEXTURES: [AdapterShape; 1] = AdapterShape::single("default", &UI_TEXTURES_ENTRIES);
// The entries of `ui.textures`, also the per-call needs of `WorldCanvas::draw`.
const UI_TEXTURES_ENTRIES: [&str; 16] = [
    "ClientInstance.getTextureGroup",
    "TextureGroup.uploadTexture",
    "TextureGroup.getTexture",
    "TextureGroup.unloadTexture",
    "TextureGroup.textureGroupBase",
    "TextureGroup.baseOffset",
    "ResourceLocation.ctor",
    "ResourceLocation.size",
    "ImageBuffer.ctorFromDescription",
    "ImageBuffer.size",
    "ImageBuffer.mData",
    "ImageBuffer.mSize",
    "ImageBuffer.mDeleter",
    "MinecraftUIRenderContext.getTexture",
    "core.operator_new",
    "core.operator_delete",
];

// The text calls: the draw and flush of the render context, the client's font handle and its destructor, the font
// vector of the repository that the handle's font id indexes, and the font's line length.
static UI_TEXT: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "MinecraftUIRenderContext.drawText",
        "MinecraftUIRenderContext.flushText",
        "ClientInstance.getFontHandle",
        "Font.getLineLength",
        "FontHandle.dtor",
        "FontHandle.size",
        "FontHandle.mDefaultFont",
        "FontRepository.mLoadedFonts",
    ],
);

// The player list handler of the client's network handler and the layout of its packet: the entries vector, the size of
// one entry, the variant index that tells Remove from Add, and the uuid, actor id, skin, name and Xbox user id of an Add
// entry.
static NETWORK_PLAYER_LIST: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.PlayerListPacket",
        "PlayerListPacket.mEntries",
        "PlayerListAddEntry.size",
        "PlayerListPacket.variantIndex",
        "PlayerListAddEntry.mUUID",
        "PlayerListAddEntry.mId",
        "PlayerListAddEntry.mSkin",
        "PlayerListAddEntry.mName",
        "PlayerListAddEntry.mXUID",
    ],
);
// The icon blit and the draw of the locator bar (a vtable slot), the vtable that holds the draw, the waypoint list of the
// local player's locator bar, the colour and visible members of a waypoint, and the actor id of an entity waypoint with
// the vtable that tells it from other waypoints.
static UI_LOCATOR_BAR_ICON: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "HudLocatorBarRenderer.vtable",
        "HudLocatorBarRenderer.render",
        "HudLocatorBarRenderer.drawIcon",
        "ClientInstance.getLocalPlayer",
        "LocalPlayer.mClientLocatorBar",
        "ClientLocatorBar.listHead",
        "ClientLocatorBar.nodeWaypoint",
        "ClientWaypoint.mColor",
        "ClientWaypoint.mColorEngaged",
        "ClientWaypoint.mVisible",
        "ClientEntityWaypoint.vtable",
        "ClientEntityWaypoint.mActorId",
    ],
);
// The chat line constructor and the layout of a `GuiMessage`: its size and the type, username, full string, filtered full
// string (with its engaged byte) and author id. One shape per constructor argument list (`GuiMessage.ctorShape`): the 70.x
// list comes first, and the first shape that resolves wins. Each constructor entry is guarded by `GuiMessage.ctorShape`:
// `GuiMessage.ctorWithFlagWord` is refused as not for this build on 26.6x, `GuiMessage.ctor` on 70.x.
static UI_CHAT_MESSAGE: [AdapterShape; 2] = [
    AdapterShape {
        name: AdapterCatalog::CHAT_FLAG_WORD_SHAPE,
        entries: &[
            "GuiMessage.ctorWithFlagWord",
            "GuiMessage.ctorShape",
            "GuiMessage.size",
            "GuiMessage.mType",
            "GuiMessage.mUsername",
            "GuiMessage.mFullString",
            "GuiMessage.mFilteredFullString",
            "GuiMessage.mFilteredFullStringEngaged",
            "GuiMessage.mAuthorXuid",
        ],
        optional: &[],
    },
    AdapterShape {
        name: AdapterCatalog::CHAT_CTOR_SHAPE,
        entries: &[
            "GuiMessage.ctor",
            "GuiMessage.ctorShape",
            "GuiMessage.size",
            "GuiMessage.mType",
            "GuiMessage.mUsername",
            "GuiMessage.mFullString",
            "GuiMessage.mFilteredFullString",
            "GuiMessage.mFilteredFullStringEngaged",
            "GuiMessage.mAuthorXuid",
        ],
        optional: &[],
    },
];
// The inline chat icons: the width of a bitmap font character and its vtable, the font handle of the client (the proof that
// the default font is a bitmap font), and the texture group calls that upload the glyph page.
static UI_INLINE_ICONS: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "BitmapFont._getCharWidth",
        "BitmapFont.vtable",
        "MinecraftUIRenderContext.drawText",
        "MinecraftUIRenderContext.flushText",
        "ClientInstance.getFontHandle",
        "Font.getLineLength",
        "FontHandle.dtor",
        "FontHandle.size",
        "FontHandle.mDefaultFont",
        "FontRepository.mLoadedFonts",
        "ClientInstance.getTextureGroup",
        "TextureGroup.uploadTexture",
        "TextureGroup.getTexture",
        "TextureGroup.unloadTexture",
        "TextureGroup.textureGroupBase",
        "TextureGroup.baseOffset",
        "ResourceLocation.ctor",
        "ResourceLocation.size",
        "ImageBuffer.ctorFromDescription",
        "ImageBuffer.size",
        "ImageBuffer.mData",
        "ImageBuffer.mSize",
        "ImageBuffer.mDeleter",
        "MinecraftUIRenderContext.getTexture",
        "core.operator_new",
        "core.operator_delete",
    ],
);
// The skin handler of the client's network handler and the uuid and skin of its packet.
static NETWORK_PLAYER_SKIN: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.PlayerSkinPacket",
        "PlayerSkinPacket.mUUID",
        "PlayerSkinPacket.mSkin",
    ],
);
// The main image of a serialized skin.
static NETWORK_SKIN: [AdapterShape; 1] = AdapterShape::single("default", &["SerializedSkinImpl.mSkinImage"]);
// The animated images of a serialized skin: the vector, the size of one element and the type and image inside it.
static NETWORK_SKIN_ANIMATED: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "SerializedSkinImpl.mSkinAnimatedImages",
        "AnimatedImageData.size",
        "AnimatedImageData.mType",
        "AnimatedImageData.mImage",
    ],
);
// The geometry of a serialized skin: the JSON value and the default name, the call that writes the JSON as text and the
// game's delete that frees the text.
static NETWORK_SKIN_GEOMETRY: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "SerializedSkinImpl.mGeometryData",
        "SerializedSkinImpl.mDefaultGeometryName",
        "JsonValue.toStyledString",
        "core.operator_delete",
    ],
);
// The persona flag, the premium flag and the resource patch text of a serialized skin.
static NETWORK_SKIN_DETAILS: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["SerializedSkinImpl.mIsPersona", "SerializedSkinImpl.mIsPremium", "SerializedSkinImpl.mResourcePatch"],
);

// The chunk cache switch: the client's `NetworkSystem::send` (hooked before the packet is serialized), the vtable that
// tells a `ClientCacheStatusPacket`, and the offset of its `mEnabled` byte.
static NETWORK_CHUNK_CACHE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["NetworkSystem.send", "ClientCacheStatusPacket.vtable", "ClientCacheStatusPacket.mEnabled"],
);
// The network protocol number of the build.
static NETWORK_PROTOCOL: [AdapterShape; 1] =
    AdapterShape::single("default", &["SharedConstants.NetworkProtocolVersion"]);

// The game clock hold: the hooked `Timer::advanceTime` and the four client timers it writes while held (the
// `MinecraftGame` pointers to its sim and real timers, and the client's own sim and real timers), the particle engine
// (the hooked tick and its valid byte, last tick time and particle count) and the predicted movement (the hooked tick
// and the `mNow` of its uniforms), which follow the held clock.
static TIME_CONTROL: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "Timer.advanceTime",
        "MinecraftGame.mSimTimer",
        "MinecraftGame.mRealTimer",
        "ClientInstance.mSimTimer",
        "ClientInstance.mRealTimer",
        "ParticleSystemEngine.tick",
        "ParticleSystemEngine.mValid",
        "ParticleSystemEngine.mLastTickTime",
        "ParticleSystemEngine.mParticleCount",
        "PredictedMovementSystem.tickPredictedMovement",
        "SystemUniforms.mNow",
    ],
);
// `FrameEnd` Before the hooked `MinecraftGame::endFrame`.
static TIME_FRAME_END: [AdapterShape; 1] = AdapterShape::single("default", &["MinecraftGame.endFrame"]);
// `LatencyBarrier` After the game's handler of a `NetworkStackLatencyPacket`: the hooked slot of the client's network
// handler.
static TIME_LATENCY_BARRIER: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["LegacyClientNetworkHandler.vtable", "ClientNetworkHandler.handle.NetworkStackLatencyPacket"],
);

// The packet wire: the client's send (`_sendInternal`) and the outer peer's receive (`PacketTraceNetworkPeer`) hooks,
// the server test of a network system, the connection walk (`mConnections`, the connection's peer and type), the receive
// slot of a peer for the coverage compare, the protocol number of the build, and the client's connection info for the host
// and port of a remote connection.
static NETWORK_PACKET_WIRE: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
        "NetworkSystem.sendInternal",
        "NetworkSystem.isServer",
        "NetworkSystem.mConnections",
        "NetworkConnection.mPeer",
        "NetworkConnection.mType",
        "PacketTraceNetworkPeer.receivePacket",
        "NetworkPeer.receivePacket",
        "SharedConstants.NetworkProtocolVersion",
        "ClientInstance.getGameConnectionInfo",
        "GameConnectionInfo.size",
        "GameConnectionInfo.dtor",
    ],
);

static NEEDS: [AdapterNeed; 154] = [
    AdapterNeed::event("frame.update", &FRAME_UPDATE),
    AdapterNeed::event("client.tick", &CLIENT_TICK),
    AdapterNeed::event_at("client.game_tick", 2, &CLIENT_GAME_TICK),
    AdapterNeed::event_at("world.join_started", 2, &JOIN_STARTED),
    AdapterNeed::event_at("world.server_join_starting", 2, &SERVER_JOIN_STARTING),
    AdapterNeed::event("world.storage_opened", &WORLD_STORAGE),
    AdapterNeed::event("world.storage_closed", &WORLD_STORAGE),
    AdapterNeed::event("world.joined", &JOINED),
    AdapterNeed::event("world.left", &WORLD_LEFT),
    AdapterNeed::event("mods.loaded", &LOADER_ONLY),
    AdapterNeed::event("registry.provider_ready", &LOADER_ONLY),
    AdapterNeed::event("registry.provider_gone", &LOADER_ONLY),
    AdapterNeed::event("loader.shutdown", &LOADER_ONLY),
    AdapterNeed::area("client.read", &CLIENT_READ),
    AdapterNeed::area_at("client.top_screen", 2, &CLIENT_TOP_SCREEN),
    AdapterNeed::area("client.hud", &CLIENT_HUD),
    AdapterNeed::area("client.mouse", &CLIENT_MOUSE),
    AdapterNeed::area("client.pause", &CLIENT_PAUSE),
    AdapterNeed::event("client.mouse_grab", &CLIENT_MOUSE),
    AdapterNeed::event("client.app_focus_lost", &CLIENT_APP_FOCUS_LOST),
    AdapterNeed::event("client.app_focus_gained", &CLIENT_APP_FOCUS_GAINED),
    AdapterNeed::area("client.focus_replay", &CLIENT_FOCUS_REPLAY),
    AdapterNeed::area("client.in_gameplay", &CLIENT_IN_GAMEPLAY),
    AdapterNeed::area("client.leave_world", &CLIENT_LEAVE_WORLD),
    AdapterNeed::area("client.can_join", &CLIENT_CAN_JOIN),
    AdapterNeed::area("client.start_local_world", &CLIENT_START_LOCAL_WORLD),
    AdapterNeed::area("client.join_server", &CLIENT_JOIN_SERVER),
    AdapterNeed::area("client.join_server_direct", &CLIENT_JOIN_SERVER_DIRECT),
    AdapterNeed::area("player.hit_result", &PLAYER_HIT_RESULT),
    AdapterNeed::event("client.join_server_result", &CLIENT_JOIN_SERVER),
    AdapterNeed::area("client.open_menu", &CLIENT_OPEN_MENU),
    AdapterNeed::area("client.go_to_title", &CLIENT_OPEN_MENU),
    AdapterNeed::area("client.send_command", &CLIENT_SEND_COMMAND),
    AdapterNeed::area("hud.select_hotbar", &HUD_SELECT_HOTBAR),
    AdapterNeed::event("input.key", &INPUT_KEY),
    AdapterNeed::area("input.key_inject", &INPUT_KEY),
    AdapterNeed::event("input.mouse", &INPUT_MOUSE),
    AdapterNeed::area("input.mouse_inject", &INPUT_MOUSE),
    AdapterNeed::event("input.look_delta", &INPUT_LOOK),
    AdapterNeed::area("input.turn", &INPUT_LOOK),
    AdapterNeed::event("input.player_move", &INPUT_MOVE),
    AdapterNeed::area("input.move", &INPUT_MOVE),
    AdapterNeed::event("input.gamepad_poll", &INPUT_GAMEPAD),
    AdapterNeed::area("input.lease", &LOADER_ONLY),
    AdapterNeed::event("input.lease_suspended", &LOADER_ONLY),
    AdapterNeed::event("input.lease_resumed", &LOADER_ONLY),
    AdapterNeed::event("input.lease_lost", &LOADER_ONLY),
    AdapterNeed::event_at("input.injected_released", 2, &LOADER_ONLY),
    AdapterNeed::area("window.read", &WINDOW_ONLY),
    AdapterNeed::area("window.request_size", &WINDOW_ONLY),
    AdapterNeed::area("window.cursor_clip", &WINDOW_ONLY),
    AdapterNeed::event("window.pointer", &WINDOW_ONLY),
    AdapterNeed::event("window.char", &WINDOW_ONLY),
    AdapterNeed::event("window.focus", &WINDOW_ONLY),
    AdapterNeed::event("window.size", &WINDOW_ONLY),
    AdapterNeed::event("window.changed", &WINDOW_ONLY),
    AdapterNeed::event("window.cursor", &WINDOW_ONLY),
    AdapterNeed::area("overlay.surface", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.frame_present", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.frame_presented", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.swap_chain_resize", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.swap_chain_resized", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.surface_lost", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.render_device_reset", &OVERLAY_ONLY),
    AdapterNeed::area("overlay.screenshot", &OVERLAY_ONLY),
    AdapterNeed::area("overlay.capture", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.screenshot_taken", &OVERLAY_ONLY),
    AdapterNeed::event("overlay.screenshot_written", &OVERLAY_ONLY),
    AdapterNeed::event("camera.fov", &CAMERA_FOV),
    AdapterNeed::event("world.dimension_changing", &DIMENSION_CHANGING),
    AdapterNeed::event("render.world_rendered", &RENDER_WORLD_RENDERED),
    AdapterNeed::area_at("render.world_canvas", 3, &RENDER_WORLD_CANVAS),
    AdapterNeed::area("render.block_mesher", &RENDER_BLOCK_MESHER),
    AdapterNeed::area("render.water_tint", &RENDER_WATER_TINT),
    AdapterNeed::event("render.block_entity_pass", &RENDER_BLOCK_ENTITY_PASS),
    AdapterNeed::area("render.block_entities", &RENDER_BLOCK_ENTITIES),
    AdapterNeed::area("render.chest_pair", &RENDER_CHEST_PAIR),
    AdapterNeed::area("render.skull_pose", &RENDER_SKULL_POSE),
    AdapterNeed::event("camera.setup", &CAMERA_SETUP),
    AdapterNeed::area("camera.pose", &CAMERA_POSE),
    AdapterNeed::area("camera.projection", &CAMERA_PROJECTION),
    AdapterNeed::area("camera.free", &CAMERA_FREE),
    AdapterNeed::area("camera.active", &CAMERA_ACTIVE),
    AdapterNeed::area("camera.hide_actor", &CAMERA_HIDE_ACTOR),
    AdapterNeed::event("player.moved", &PLAYER_MOVED),
    AdapterNeed::event("camera.instruction_received", &CAMERA_INSTRUCTION_RECEIVED),
    AdapterNeed::area_at("actor.read", 4, &ACTOR_READ),
    AdapterNeed::area("actor.health", &ACTOR_HEALTH),
    AdapterNeed::area("actor.game_type", &ACTOR_GAME_TYPE),
    AdapterNeed::area("actor.permission", &ACTOR_PERMISSION),
    AdapterNeed::area("actor.look_write", &ACTOR_LOOK_WRITE),
    AdapterNeed::area("world.sky_darken", &WORLD_SKY_DARKEN),
    AdapterNeed::area("world.chunk_read", &WORLD_CHUNK_READ),
    AdapterNeed::area("world.map_color", &WORLD_MAP_COLOR),
    AdapterNeed::area("world.biomes", &WORLD_BIOMES),
    AdapterNeed::area("world.scan_surfaces", &WORLD_SCAN_SURFACES),
    AdapterNeed::area("world.shapes", &WORLD_SHAPES),
    AdapterNeed::area("world.raycast", &WORLD_RAYCAST),
    AdapterNeed::area("world.actors_in", &WORLD_ACTORS_IN),
    AdapterNeed::area("world.block_states", &WORLD_BLOCK_STATES),
    AdapterNeed::area("world.blocks", &WORLD_BLOCKS),
    AdapterNeed::area("world.kinds_in_box", &WORLD_KINDS_IN_BOX),
    AdapterNeed::event_at("world.block_changed", 2, &BLOCK_CHANGED),
    AdapterNeed::area("client.world_info", &CLIENT_WORLD_INFO),
    AdapterNeed::area_at("inventory.read", 4, &INVENTORY_READ),
    AdapterNeed::area_at("inventory.request", 2, &INVENTORY_REQUEST_PICK),
    AdapterNeed::event("inventory.request_done", &INVENTORY_REQUEST),
    AdapterNeed::area("lighting.terrain", &LIGHTING_TERRAIN),
    AdapterNeed::area("lighting.entity", &LIGHTING_ENTITY),
    AdapterNeed::area("lighting.vibrant", &LIGHTING_VIBRANT),
    AdapterNeed::event("player.block_break_start", &BLOCK_BREAK_START),
    AdapterNeed::event("player.block_break_continue", &BLOCK_BREAK_CONTINUE),
    AdapterNeed::event("player.item_use", &ITEM_USE),
    AdapterNeed::event("player.attack", &PLAYER_ATTACK),
    AdapterNeed::event("player.item_use_on_block", &ITEM_USE_ON_BLOCK),
    AdapterNeed::event("player.use_pressed", &USE_PRESSED),
    AdapterNeed::event("player.build_block", &BUILD_BLOCK),
    AdapterNeed::area("player.build", &PLAYER_BUILD),
    AdapterNeed::area("player.pick_range", &PLAYER_PICK_RANGE),
    AdapterNeed::area("client.activity", &CLIENT_ACTIVITY),
    AdapterNeed::area("options.view_distance", &OPTIONS_VIEW_DISTANCE),
    AdapterNeed::area("options.int", &OPTIONS_INT),
    AdapterNeed::event("client.idle_screen_open", &IDLE_SCREEN_OPEN),
    AdapterNeed::area("client.title_screen", &IDLE_SCREEN_OPEN),
    AdapterNeed::event("ui.screen_render", &UI_SCREEN_RENDER),
    AdapterNeed::area_at("ui.screen", 2, &UI_SCREEN),
    AdapterNeed::event("ui.screen_closing", &UI_SCREEN_CLOSING),
    AdapterNeed::event("ui.screen_changed", &UI_SCREEN_CHANGED),
    AdapterNeed::event("ui.container_slot_hover", &UI_CONTAINER_SLOT_HOVER),
    AdapterNeed::event("ui.container_slot_unhover", &UI_CONTAINER_SLOT_UNHOVER),
    AdapterNeed::event("input.ui_pointer", &INPUT_UI_POINTER),
    AdapterNeed::event("input.ui_scroll", &INPUT_UI_SCROLL),
    AdapterNeed::event("ui.item_hovertext", &UI_ITEM_HOVERTEXT),
    AdapterNeed::area_at("ui.draw", 2, &UI_DRAW),
    AdapterNeed::area("ui.textures", &UI_TEXTURES),
    AdapterNeed::event("ui.textures_reloaded", &UI_TEXTURES_RELOADED),
    AdapterNeed::area("ui.text", &UI_TEXT),
    AdapterNeed::event("network.player_list", &NETWORK_PLAYER_LIST),
    AdapterNeed::event("network.player_skin", &NETWORK_PLAYER_SKIN),
    AdapterNeed::area("network.skin", &NETWORK_SKIN),
    AdapterNeed::area("network.skin_animated", &NETWORK_SKIN_ANIMATED),
    AdapterNeed::area("network.skin_geometry", &NETWORK_SKIN_GEOMETRY),
    AdapterNeed::area("network.skin_details", &NETWORK_SKIN_DETAILS),
    AdapterNeed::area("network.chunk_cache", &NETWORK_CHUNK_CACHE),
    AdapterNeed::area("network.protocol", &NETWORK_PROTOCOL),
    AdapterNeed::event("network.packet_wire", &NETWORK_PACKET_WIRE),
    AdapterNeed::event("network.connection_started", &NETWORK_PACKET_WIRE),
    AdapterNeed::area("time.control", &TIME_CONTROL),
    AdapterNeed::event("time.step_applied", &TIME_CONTROL),
    AdapterNeed::event("time.frame_end", &TIME_FRAME_END),
    AdapterNeed::event("time.latency_barrier", &TIME_LATENCY_BARRIER),
    AdapterNeed::event("ui.locator_bar_icon", &UI_LOCATOR_BAR_ICON),
    AdapterNeed::event("ui.chat_message", &UI_CHAT_MESSAGE),
    AdapterNeed::area("ui.inline_icons", &UI_INLINE_ICONS),
];

/// The static list of game entries each API event and area needs.
pub struct AdapterCatalog;

impl AdapterCatalog {
    /// The vtable global that holds the `MinecraftGame.update` slot.
    pub const MINECRAFT_GAME_VTABLE: &'static str = "MinecraftGame.vtable";
    /// The vtable global that holds the `ClientInstance.onStartJoinGame` slot.
    pub const CLIENT_INSTANCE_VTABLE: &'static str = "ClientInstance.vtable";
    /// The shape of the needs the loader's window subclass serves.
    pub const WINDOW_SHAPE: &'static str = "window";
    /// The shape of the needs the loader's overlay hooks serve.
    pub const OVERLAY_SHAPE: &'static str = "overlay";
    /// The `ui.chat_message` shape of a build whose `GuiMessage` constructor has the 26.6x argument list.
    pub const CHAT_CTOR_SHAPE: &'static str = "ctor";
    /// The `ui.chat_message` shape of a build whose `GuiMessage` constructor has the 70.x argument list.
    pub const CHAT_FLAG_WORD_SHAPE: &'static str = "ctor_flag_word";
    /// The `lighting.vibrant` shape of 26.6x: the grid fill with a mode argument.
    pub const LIGHT_GRID_SHAPE: &'static str = "fillSubChunkGrid";
    /// The `lighting.vibrant` shape of 70.x: the grid fill without a mode argument.
    pub const LIGHT_GRID_NO_MODE_SHAPE: &'static str = "fillSubChunkGridNoMode";
    /// The `world.map_color` shape of 26.63: the map colour component of the block, tinted by its biome policy.
    pub const MAP_COLOR_COMPONENT_SHAPE: &'static str = "component";
    /// The `world.map_color` shape of 70.x: `Block::getMapColor`.
    pub const MAP_COLOR_BLOCK_SHAPE: &'static str = "block";
    /// The `camera.active` shape of 26.63: the `CameraComponent` size built from a factor and a shift.
    pub const ACTIVE_CAMERA_FACTOR_SHAPE: &'static str = "factor_shift";
    /// The `camera.active` shape of 70.x: the `CameraComponent` size as one value.
    pub const ACTIVE_CAMERA_SIZE_SHAPE: &'static str = "size";
    /// The `world.scan_surfaces` shape of 26.63: `Block::canProvideFullSupport(face)`.
    pub const SURFACE_SUPPORT_SHAPE: &'static str = "support";
    /// The `world.scan_surfaces` shape of 70.x: `Block::canProvideFullSupport(face, flag)`.
    pub const SURFACE_SUPPORT_FLAG_SHAPE: &'static str = "support_flag";
    /// The `actor.permission` shape of 26.63: `Player::getPlayerPermissionLevel`.
    pub const PERMISSION_GETTER_SHAPE: &'static str = "getter";
    /// The `actor.permission` shape of 70.x: the byte of the abilities component of `Player::getAbilitiesComponent`.
    pub const PERMISSION_ABILITIES_SHAPE: &'static str = "abilities";
    /// The `player.pick_range` shape of 26.63: `GameMode::getMaxPickRange` on the player's game mode.
    pub const PICK_RANGE_GETTER_SHAPE: &'static str = "getter";
    /// The `player.pick_range` shape of 70.x: `Player::isCreative` picks one of the game's two range constants.
    pub const PICK_RANGE_CREATIVE_SHAPE: &'static str = "creative";
    /// The `render.block_entities` shape of 26.63: `VanillaBlockActorFactory::createBlockActor(pos, type)`.
    pub const BLOCK_ENTITIES_FACTORY_SHAPE: &'static str = "factory";
    /// The `render.block_entities` shape of 70.x: the factory takes the byte of the level's flag slot.
    pub const BLOCK_ENTITIES_FLAG_SHAPE: &'static str = "factory_flag";
    /// The `render.skull_pose` shape of 26.63: `SkullBlockRenderer::renderSkull` with the face and the turn.
    pub const SKULL_RENDER_SHAPE: &'static str = "render_skull";
    /// The `render.skull_pose` shape of 70.x: `renderSkullUpright` inside a pushed face and turn transform.
    pub const SKULL_UPRIGHT_SHAPE: &'static str = "upright";

    pub fn needs() -> &'static [AdapterNeed] {
        &NEEDS
    }

    /// The highest version of event or area `id` this catalog serves; `None` for an id it does not know. A mod row
    /// with a higher version needs a newer loader.
    pub fn served_version(kind: NeedKind, id: &str) -> Option<u32> {
        NEEDS.iter().filter(|need| need.kind == kind && need.id == id).map(|need| need.version).max()
    }

    /// One row per event or area: the first shape whose entries all resolve in `entries`, or the missing entries of the first shape.
    pub fn availability(resolved: &impl Fn(&str) -> bool) -> Vec<Availability> {
        NEEDS
            .iter()
            .map(|need| {
                let unresolved = |shape: &AdapterShape| -> Vec<&'static str> {
                    shape.entries.iter().copied().filter(|name| !resolved(name)).collect()
                };
                let held = need.shapes.iter().find(|shape| unresolved(shape).is_empty());
                let missing = match (held, need.shapes.first()) {
                    (None, Some(first)) => unresolved(first),
                    _ => Vec::new(),
                };
                Availability {
                    id: need.id,
                    kind: need.kind,
                    version: need.version,
                    shape: held.map(|shape| shape.name),
                    missing,
                }
            })
            .collect()
    }
}

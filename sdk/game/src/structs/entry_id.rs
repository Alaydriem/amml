/// The id of one entry in the per-build data file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntryId(&'static str);

impl EntryId {
    /// An id outside the shared list.
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    /// The id text, as the data file spells it.
    pub const fn as_str(&self) -> &'static str {
        self.0
    }

    pub const ACTOR_STATE_VECTOR_COMPONENT: Self = Self("Actor.stateVectorComponent");
    pub const ACTOR_ACTOR_ROTATION_COMPONENT: Self = Self("Actor.actorRotationComponent");
    pub const STATE_VECTOR_COMPONENT_POS: Self = Self("StateVectorComponent.mPos");
    pub const ACTOR_ROTATION_COMPONENT_ROT: Self = Self("ActorRotationComponent.mRot");
    pub const ACTOR_LEVEL: Self = Self("Actor.mLevel");
    pub const ACTOR_GET_DIMENSION: Self = Self("Actor.getDimension");
    pub const DIMENSION_GET_DIMENSION_ID: Self = Self("Dimension.getDimensionId");
    pub const ACTOR_GET_DIMENSION_BLOCK_SOURCE: Self = Self("Actor.getDimensionBlockSource");
    pub const LEVEL_RENDERER_PLAYER_SETUP_CAMERA: Self = Self("LevelRendererPlayer.setupCamera");
    pub const LEVEL_RENDERER_PLAYER_CREATE_VIEW_RENDER_OBJECT: Self =
        Self("LevelRendererPlayer.createViewRenderObject");
    pub const CAMERA_FORWARD: Self = Self("Camera.mForward");
    pub const VIEW_RENDER_OBJECT_VIEW_DATA: Self = Self("ViewRenderObject.mViewData");
    pub const VIEW_RENDER_DATA_CAMERA_POS: Self = Self("ViewRenderData.mCameraPos");
    pub const CLIENT_INSTANCE_UPDATE: Self = Self("ClientInstance.update");
    pub const CLIENT_INSTANCE_GET_LOCAL_PLAYER: Self = Self("ClientInstance.getLocalPlayer");
    pub const CLIENT_INSTANCE_IS_WORLD_ACTIVE: Self = Self("ClientInstance.isWorldActive");
    pub const CLIENT_INSTANCE_IS_SHOWING_MENU: Self = Self("ClientInstance.isShowingMenu");
    pub const CLIENT_INSTANCE_GET_MOUSE_GRABBED: Self = Self("ClientInstance.getMouseGrabbed");
    pub const CLIENT_INSTANCE_GET_OPTIONS: Self = Self("ClientInstance.getOptions");
    pub const I_OPTION_REGISTRY_GET_HIDE_HUD: Self = Self("IOptionRegistry.getHideHud");
    pub const CLIENT_INSTANCE_GET_REGION: Self = Self("ClientInstance.getRegion");
    pub const ACTOR_IS_ALIVE: Self = Self("Actor.isAlive");
    pub const BLOCK_SOURCE_VTABLE: Self = Self("BlockSource.vtable");
    pub const BLOCK_SOURCE_GET_CHUNK: Self = Self("BlockSource.getChunk");
    /// The offset of `mSubChunks` in `LevelChunk`; its value selects the `LevelChunk` mirror.
    pub const LEVEL_CHUNK_SUB_CHUNKS: Self = Self("LevelChunk.mSubChunks");
    pub const LEVEL_CHUNK_GET_BLOCK: Self = Self("LevelChunk.getBlock");
    /// `sizeof(SubChunk)`, the stride of `LevelChunk::mSubChunks`.
    pub const SUB_CHUNK_SIZE: Self = Self("SubChunk.size");
    pub const SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK: Self = Self("SubChunk.isPlaceHolderSubChunk");
    /// Which `BlockType` mirror the build has: 0 the Material pointer, 1 the Material by value (`MaterialLayout`).
    pub const BLOCK_TYPE_MATERIAL_INLINE: Self = Self("BlockType.mMaterialInline");
    /// `BrightnessPair BlockSource::getBrightnessPair(BlockPos const&) const`, a function address.
    pub const BLOCK_SOURCE_GET_BRIGHTNESS_PAIR: Self = Self("BlockSource.getBrightnessPair");
    /// `bool Block::canProvideFullSupport(uchar face) const`, a function address.
    pub const BLOCK_CAN_PROVIDE_FULL_SUPPORT: Self = Self("Block.canProvideFullSupport");
    /// The slot of `BlockType::getCollisionShape` in the `BlockType` vtable.
    pub const BLOCK_TYPE_GET_COLLISION_SHAPE: Self = Self("BlockType.getCollisionShape");
    /// `Vec3 Actor::getAttachPos(ActorLocation, float) const`, a function address.
    pub const ACTOR_GET_ATTACH_POS: Self = Self("Actor.getAttachPos");
    /// The offset of the `EntityContext` held by value in `Actor`.
    pub const ACTOR_ENTITY_CONTEXT: Self = Self("Actor.mEntityContext");
    /// `int ActorAttribute::getHealth(EntityContext const&)`, a function address.
    pub const ACTOR_ATTRIBUTE_GET_HEALTH: Self = Self("ActorAttribute.getHealth");
    /// `void mce::Camera::updateViewMatrixDependencies()`, a function address.
    pub const CAMERA_UPDATE_VIEW_MATRIX_DEPENDENCIES: Self = Self("Camera.updateViewMatrixDependencies");
    /// `void LevelRendererCamera::preRenderUpdate(ScreenContext&, LevelRenderPreRenderUpdateParameters&)`, the
    /// address of the base function (not a slot).
    pub const LEVEL_RENDERER_CAMERA_PRE_RENDER_UPDATE: Self = Self("LevelRendererCamera.preRenderUpdate");
    pub const LEVEL_RENDERER_CAMERA_CAMERA_POS: Self = Self("LevelRendererCamera.mCameraPos");
    pub const LEVEL_RENDERER_CAMERA_CAMERA_TARGET_POS: Self = Self("LevelRendererCamera.mCameraTargetPos");
    pub const CORE_OPERATOR_NEW: Self = Self("core.operator_new");
    pub const CORE_OPERATOR_DELETE: Self = Self("core.operator_delete");
    pub const NETWORK_SYSTEM_SEND: Self = Self("NetworkSystem.send");
    pub const NETWORK_SYSTEM_SEND_INTERNAL: Self = Self("NetworkSystem.sendInternal");
    pub const NETWORK_SYSTEM_IS_SERVER: Self = Self("NetworkSystem.isServer");
    pub const NETWORK_SYSTEM_CONNECTIONS: Self = Self("NetworkSystem.mConnections");
    pub const NETWORK_CONNECTION_PEER: Self = Self("NetworkConnection.mPeer");
    pub const NETWORK_PEER_RECEIVE_PACKET: Self = Self("NetworkPeer.receivePacket");
    pub const ACTOR_SET_Y_HEAD_ROTATIONS: Self = Self("Actor.setYHeadRotations");
    pub const CLIENT_NETWORK_HANDLER_HANDLE_MOVE_PLAYER_PACKET: Self =
        Self("ClientNetworkHandler.handle.MovePlayerPacket");
    pub const CLIENT_NETWORK_HANDLER_HANDLE_NETWORK_STACK_LATENCY_PACKET: Self =
        Self("ClientNetworkHandler.handle.NetworkStackLatencyPacket");
    pub const MINECRAFT_GAME_UPDATE: Self = Self("MinecraftGame.update");
    pub const MINECRAFT_GAME_ON_APP_FOCUS_GAINED: Self = Self("MinecraftGame.onAppFocusGained");
    pub const MINECRAFT_GAME_IS_IN_GAME: Self = Self("MinecraftGame.isInGame");
    pub const MINECRAFT_GAME_IS_PRIMARY_CLIENT_INSTANCE_READY: Self =
        Self("MinecraftGame.isPrimaryClientInstanceReady");
    pub const CLIENT_INSTANCE_ON_START_JOIN_GAME: Self = Self("ClientInstance.onStartJoinGame");
    pub const CLIENT_INSTANCE_START_EXTERNAL_NETWORK_WORLD: Self = Self("ClientInstance.startExternalNetworkWorld");
    pub const GAME_CONNECTION_INFO_CTOR: Self = Self("GameConnectionInfo.ctor");
    pub const INT_OPTION_SET: Self = Self("IntOption.set");
    pub const ORE_UI_ROUTER_PUSH_ROUTE: Self = Self("OreUIRouter.pushRoute");
    pub const ORE_UI_ROUTER_REPLACE_ROUTE: Self = Self("OreUIRouter.replaceRoute");
    pub const OPTION_REGISTRY_GET_EFFECTIVE_MAX_FRAMERATE_OPTION: Self =
        Self("OptionRegistry.getEffectiveMaxFramerateOption");
    pub const OPTION_REGISTRY_GET_IF_VALID: Self = Self("OptionRegistry.getIfValid");
    pub const TIMER_ADVANCE_TIME: Self = Self("Timer.advanceTime");
    pub const MINECRAFT_GAME_SIM_TIMER: Self = Self("MinecraftGame.mSimTimer");
    pub const MINECRAFT_GAME_REAL_TIMER: Self = Self("MinecraftGame.mRealTimer");
    pub const MINECRAFT_GAME_END_FRAME: Self = Self("MinecraftGame.endFrame");
    pub const PACKET_TRACE_NETWORK_PEER_RECEIVE_PACKET: Self = Self("PacketTraceNetworkPeer.receivePacket");
    pub const CLIENT_INSTANCE_REQUEST_LEAVE_GAME: Self = Self("ClientInstance.requestLeaveGame");
    pub const CLIENT_INSTANCE_GET_MINECRAFT_INPUT: Self = Self("ClientInstance.getMinecraftInput");
    pub const PARTICLE_SYSTEM_ENGINE_TICK: Self = Self("ParticleSystemEngine.tick");
    pub const PREDICTED_MOVEMENT_SYSTEM_TICK_PREDICTED_MOVEMENT: Self =
        Self("PredictedMovementSystem.tickPredictedMovement");
    pub const LEVEL_RENDERER_PLAYER_QUEUE_RENDER_ENTITIES: Self = Self("LevelRendererPlayer.queueRenderEntities");
    pub const LEVEL_RENDERER_CAMERA_ACTOR_RENDER_QUEUE: Self = Self("LevelRendererCamera.mActorRenderQueue");
    pub const ACTOR_RENDER_DISPATCHER_RENDER: Self = Self("ActorRenderDispatcher.render");
    /// `void ScreenView::render(UIRenderContext&)`, a function address; the UI render point.
    pub const SCREEN_VIEW_RENDER: Self = Self("ScreenView.render");
    /// The offset of `mPointerLocationPrevious` (a `glm::vec2`) in `ScreenView`.
    pub const SCREEN_VIEW_POINTER_LOCATION_PREVIOUS: Self = Self("ScreenView.mPointerLocationPrevious");
    pub const UI_SCENE_VTABLE: Self = Self("UIScene.vtable");
    /// `void UIScene::handleDirection(DirectionId, float, float, FocusImpact)`, a function address; the UI wheel point.
    pub const UI_SCENE_HANDLE_DIRECTION: Self = Self("UIScene.handleDirection");
    /// The offset of the `ScreenView` pointer in `UIScene`.
    pub const UI_SCENE_SCREEN_VIEW: Self = Self("UIScene.mScreenView");
    /// The slot of `ScreenController::_isContainerScreen` in the `ScreenController` vtable.
    pub const SCREEN_CONTROLLER_IS_CONTAINER_SCREEN: Self = Self("ScreenController.isContainerScreen");
    /// `ScreenController::~ScreenController`, a function address.
    pub const SCREEN_CONTROLLER_DTOR: Self = Self("ScreenController.dtor");
    pub const CONTAINER_SCREEN_CONTROLLER_VTABLE: Self = Self("ContainerScreenController.vtable");
    /// `_onContainerSlotHovered(std::string const&, int)`, a function address.
    pub const CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_HOVERED: Self =
        Self("ContainerScreenController.onContainerSlotHovered");
    /// `_onContainerSlotUnhovered(std::string const&, int)`, a function address.
    pub const CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED: Self =
        Self("ContainerScreenController.onContainerSlotUnhovered");
    /// The slot of `_getVisualItemStackImpl(std::string const&, int)` in the `ContainerScreenController` vtable.
    pub const CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL: Self =
        Self("ContainerScreenController.getVisualItemStackImpl");
    pub const SHULKER_BOX_BLOCK_ITEM_VTABLE: Self = Self("ShulkerBoxBlockItem.vtable");
    /// The shulker box override of `Item::appendFormattedHovertext`, a function address.
    pub const SHULKER_BOX_BLOCK_ITEM_APPEND_FORMATTED_HOVERTEXT: Self =
        Self("ShulkerBoxBlockItem.appendFormattedHovertext");
    /// The base `Item::appendFormattedHovertext`, a function address.
    pub const ITEM_APPEND_FORMATTED_HOVERTEXT: Self = Self("Item.appendFormattedHovertext");
    /// The slot of `Item::getAnimationFrameFor(Mob*, bool, ItemStack const*, bool)` in the `Item` vtable.
    pub const ITEM_GET_ANIMATION_FRAME_FOR: Self = Self("Item.getAnimationFrameFor");
    /// The slot of `Item::isGlint(ItemStackBase const&)` in the `Item` vtable.
    pub const ITEM_IS_GLINT: Self = Self("Item.isGlint");
    /// The slot of `Item::getMaxDamage()` in the `Item` vtable.
    pub const ITEM_GET_MAX_DAMAGE: Self = Self("Item.getMaxDamage");
    /// The offset of `mFullName` (a `HashedString`) in `Item`.
    pub const ITEM_FULL_NAME: Self = Self("Item.mFullName");
    /// `static ItemStack ItemStack::fromTag(CompoundTag const&)`, a function address.
    pub const ITEM_STACK_FROM_TAG: Self = Self("ItemStack.fromTag");
    /// `short ItemStackBase::getDamageValue() const`, a function address.
    pub const ITEM_STACK_BASE_GET_DAMAGE_VALUE: Self = Self("ItemStackBase.getDamageValue");
    /// `bool ItemStackBase::isDamaged() const`, a function address.
    pub const ITEM_STACK_BASE_IS_DAMAGED: Self = Self("ItemStackBase.isDamaged");
    /// The offset of `std::unique_ptr<PlayerInventory> Player::mInventory` in `Player`.
    pub const PLAYER_PLAYER_INVENTORY: Self = Self("Player.playerInventory");
    /// The offset of `std::vector<ItemStack> FillingContainer::mItems` (its first pointer) in `FillingContainer`.
    pub const FILLING_CONTAINER_ITEMS: Self = Self("FillingContainer.mItems");
    /// The base `GameMode` vtable (no RTTI).
    pub const GAME_MODE_VTABLE: Self = Self("GameMode.vtable");
    /// The offset of `Player& GameMode::mPlayer` in `GameMode`.
    pub const GAME_MODE_M_PLAYER: Self = Self("GameMode.mPlayer");
    /// The slot of `bool startDestroyBlock(BlockPos const&, uchar, bool&)` in the `GameMode` vtable.
    pub const GAME_MODE_START_DESTROY_BLOCK: Self = Self("GameMode.startDestroyBlock");
    /// The slot of `bool continueDestroyBlock(BlockPos const&, uchar, Vec3 const&, bool&)` in the `GameMode` vtable.
    pub const GAME_MODE_CONTINUE_DESTROY_BLOCK: Self = Self("GameMode.continueDestroyBlock");
    /// The slot of `bool attack(Actor&, Vec3 const&)` in the `GameMode` vtable. The `SurvivalMode` table holds its
    /// own leaf in this slot, which does not run the base slot.
    pub const GAME_MODE_ATTACK: Self = Self("GameMode.attack");
    /// The slot of `bool useItem(ItemStack&, bool)` in the `GameMode` vtable.
    pub const GAME_MODE_USE_ITEM: Self = Self("GameMode.useItem");
    /// The vtable of `SurvivalMode`, the `GameMode` subclass of a player in survival (no RTTI).
    pub const SURVIVAL_MODE_VTABLE: Self = Self("SurvivalMode.vtable");
    /// `InteractionResult Item::useOn(ItemStack&, Actor&, BlockPos const&, uchar, uchar, Vec3 const&, u64) const`,
    /// a function address; the result comes back through a hidden pointer.
    pub const ITEM_USE_ON: Self = Self("Item.useOn");
    /// `ItemRenderer::renderGuiItemNew`, a function address.
    pub const ITEM_RENDERER_RENDER_GUI_ITEM_NEW: Self = Self("ItemRenderer.renderGuiItemNew");
    /// The constructor of `BaseActorRenderContext`, a function address.
    pub const BASE_ACTOR_RENDER_CONTEXT_CTOR: Self = Self("BaseActorRenderContext.ctor");
    /// The (complete) destructor of `BaseActorRenderContext`, a function address.
    pub const BASE_ACTOR_RENDER_CONTEXT_DTOR: Self = Self("BaseActorRenderContext.dtor");
    /// `sizeof(BaseActorRenderContext)`.
    pub const BASE_ACTOR_RENDER_CONTEXT_SIZE: Self = Self("BaseActorRenderContext.size");
    pub const MINECRAFT_UI_RENDER_CONTEXT_VTABLE: Self = Self("MinecraftUIRenderContext.vtable");
    /// The offset of `mScreenContext` in `MinecraftUIRenderContext`.
    pub const MINECRAFT_UI_RENDER_CONTEXT_M_SCREEN_CONTEXT: Self = Self("MinecraftUIRenderContext.mScreenContext");
    /// The slot of `fillRectangle` in the `MinecraftUIRenderContext` vtable.
    pub const MINECRAFT_UI_RENDER_CONTEXT_FILL_RECTANGLE: Self = Self("MinecraftUIRenderContext.fillRectangle");
    /// The slot of `drawRectangle` in the `MinecraftUIRenderContext` vtable.
    pub const MINECRAFT_UI_RENDER_CONTEXT_DRAW_RECTANGLE: Self = Self("MinecraftUIRenderContext.drawRectangle");
    /// The slot of `drawText` (the `std::string_view` form) in the `MinecraftUIRenderContext` vtable.
    pub const MINECRAFT_UI_RENDER_CONTEXT_DRAW_TEXT: Self = Self("MinecraftUIRenderContext.drawText");
    /// The slot of `flushText` in the `MinecraftUIRenderContext` vtable.
    pub const MINECRAFT_UI_RENDER_CONTEXT_FLUSH_TEXT: Self = Self("MinecraftUIRenderContext.flushText");
    /// The slot of `getMinecraftGame_DEPRECATED` in the `ClientInstance` vtable.
    pub const CLIENT_INSTANCE_GET_MINECRAFT_GAME: Self = Self("ClientInstance.getMinecraftGame");
    /// The offset of the variant index byte in a `CompoundTag` map node.
    pub const COMPOUND_TAG_NODE_VARIANT_INDEX: Self = Self("CompoundTag.nodeVariantIndex");
}

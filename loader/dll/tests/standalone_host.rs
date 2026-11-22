use amml::{LoaderLog, LoaderState, StandaloneHost};
use amml_data::EntryId;

use crate::support::{LogFile, Probe};

fn id(name: &str) -> EntryId {
    EntryId(name.to_string())
}

#[test]
fn the_standalone_host_also_wants_the_entries_of_the_session_events() {
    let wanted: Vec<String> = StandaloneHost::wanted(&[]).into_iter().map(|EntryId(id)| id).collect();

    assert_eq!(
        wanted,
        [
            "AbilitiesComponent.mPlayerPermissions",
            "ActiveCameraComponent.typeId",
            "Actor.aabbShapeComponent",
            "Actor.actorRotationComponent",
            "Actor.getAttachPos",
            "Actor.getDimension",
            "Actor.getItemInHandSlot",
            "Actor.getRuntimeID",
            "Actor.getViewVector",
            "Actor.isAlive",
            "Actor.mEntityContext",
            "Actor.mLevel",
            "Actor.setYHeadRotations",
            "Actor.stateVectorComponent",
            "ActorAttribute.getHealth",
            "ActorRenderDispatcher.render",
            "ActorRotationComponent.mRot",
            "ActorShaderManager.setEntityConstants",
            "AnimatedImageData.mImage",
            "AnimatedImageData.mType",
            "AnimatedImageData.size",
            "BaseActorRenderContext.Impl.mCameraTargetPosition",
            "BaseActorRenderContext.ctor",
            "BaseActorRenderContext.dtor",
            "BaseActorRenderContext.mBlockEntityRenderDispatcher",
            "BaseActorRenderContext.mImpl",
            "BaseActorRenderContext.size",
            "BaseActorRenderer.ctor",
            "BaseActorRenderer.size",
            "Biome.mHash",
            "Biome.mId",
            "BiomeColorSampling.getMapGrassColor",
            "BiomeColorSampling.getMapPolicy",
            "BiomeColorSampling.getTessellationPolicy",
            "BiomeColorSampling.getWaterColor",
            "BitmapFont._getCharWidth",
            "BitmapFont.vtable",
            "Block.canProvideFullSupport",
            "Block.canProvideFullSupportWithFlag",
            "Block.getMapColor",
            "Block.getRenderLayer",
            "Block.mBlockType",
            "Block.mDirectData.mLightEmission",
            "Block.mSerializationId",
            "Block.tryGetComponent.BlockMapColorComponent",
            "BlockActor._getRenderComponent",
            "BlockActor.isType",
            "BlockActor.loadStatic",
            "BlockActor.mPosition",
            "BlockActorRenderDispatcher.mSkullRenderer",
            "BlockActorRenderDispatcher.render",
            "BlockGraphics.getForBlock",
            "BlockGraphics.mBlockShape",
            "BlockSerializationUtils.tryGetBlockFromNBT",
            "BlockSource.addListener",
            "BlockSource.clip",
            "BlockSource.fetchEntities",
            "BlockSource.fetchEntitiesSpanOrder",
            "BlockSource.getBlock",
            "BlockSource.getBrightnessPair",
            "BlockSource.getChunk",
            "BlockSource.getDimensionId",
            "BlockSource.getLightColor",
            "BlockSource.removeListener",
            "BlockSourceListener.onBlockChanged",
            "BlockSourceListener.onSourceDestroyed",
            "BlockSourceListener.vtable",
            "BlockTessellator.ctor",
            "BlockTessellator.dtor",
            "BlockTessellator.mBlockCache",
            "BlockTessellator.setRegion",
            "BlockTessellator.tessellateInWorld",
            "BlockTessellatorCache.getLightColor",
            "BlockType.asItemInstance",
            "BlockType.canBeBuiltOver",
            "BlockType.getCollisionShape",
            "BlockType.getExtraRenderLayers",
            "BlockType.getMapColor",
            "BlockType.isInteractiveBlock",
            "BlockType.mMaterialInline",
            "BlockType.mNameInfo.mFullName",
            "BlockType.mTintMethod",
            "BlockTypeRegistry.instance",
            "BlockTypeRegistry.lookupByName",
            "Camera.mForward",
            "Camera.updateViewMatrixDependencies",
            "CameraComponent.size",
            "CameraComponent.sizeFactor",
            "CameraComponent.sizeShift",
            "CameraComponent.typeId",
            "CameraInstruction.mSpline",
            "CameraInstructionPacket.mCameraInstruction",
            "CameraInstructionPacket.size",
            "CameraPreset.size",
            "CameraRegistry.cameraEntityStride",
            "CameraRegistry.mCameraEntities",
            "ChestBlockActor.mLargeChestPaired",
            "ChestBlockActor.mLargeChestPairedPosition",
            "ChestBlockActor.mPairFlags",
            "ClientCacheStatusPacket.mEnabled",
            "ClientCacheStatusPacket.vtable",
            "ClientEntityWaypoint.mActorId",
            "ClientEntityWaypoint.vtable",
            "ClientExitLevel.hook",
            "ClientInputCallbacks.handleBuildAction",
            "ClientInputUpdateSystem.extractRawHIDInput",
            "ClientInstance.connectByAddress",
            "ClientInstance.connectByAddressThis",
            "ClientInstance.gameCallbacksOffset",
            "ClientInstance.getCameraRegistry",
            "ClientInstance.getCurrentUIRouter",
            "ClientInstance.getFontHandle",
            "ClientInstance.getGameConnectionInfo",
            "ClientInstance.getItemInHandRenderer",
            "ClientInstance.getLevel",
            "ClientInstance.getLocalPlayer",
            "ClientInstance.getMinecraftGame",
            "ClientInstance.getMinecraftInput",
            "ClientInstance.getOptions",
            "ClientInstance.getPacketSender",
            "ClientInstance.getTextureGroup",
            "ClientInstance.getTopScreenName",
            "ClientInstance.isInRealm",
            "ClientInstance.isShowingMenu",
            "ClientInstance.isWorldActive",
            "ClientInstance.mRealTimer",
            "ClientInstance.mSimTimer",
            "ClientInstance.onStartJoinGame",
            "ClientInstance.onTick",
            "ClientInstance.requestLeaveGame",
            "ClientInstance.startExternalNetworkWorld",
            "ClientInstance.update",
            "ClientInstance.vtable",
            "ClientLevel.getLevelData",
            "ClientLocatorBar.listHead",
            "ClientLocatorBar.nodeWaypoint",
            "ClientNetworkHandler.handle.CameraInstructionPacket",
            "ClientNetworkHandler.handle.ContainerClosePacket",
            "ClientNetworkHandler.handle.ContainerOpenPacket",
            "ClientNetworkHandler.handle.ItemStackResponsePacket",
            "ClientNetworkHandler.handle.MovePlayerPacket",
            "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket",
            "ClientNetworkHandler.handle.NetworkStackLatencyPacket",
            "ClientNetworkHandler.handle.PlayerListPacket",
            "ClientNetworkHandler.handle.PlayerSkinPacket",
            "ClientNetworkHandler.handle.PrimitiveShapesPacket",
            "ClientWaypoint.mColor",
            "ClientWaypoint.mColorEngaged",
            "ClientWaypoint.mVisible",
            "CommandRequestPacket.mCommand",
            "CommandRequestPacket.size",
            "CompoundTag.dtor",
            "CompoundTag.nodeVariantIndex",
            "CompoundTag.vtable",
            "CompoundTag.write",
            "ContainerManagerController.handleSwap",
            "ContainerScreenController._handleAutoPlace",
            "ContainerScreenController._handleDropItem",
            "ContainerScreenController._handleDropSelectedItem",
            "ContainerScreenController._handlePlaceAll",
            "ContainerScreenController._handleTakeAll",
            "ContainerScreenController.getVisualItemStackImpl",
            "ContainerScreenController.mContainerManagerController",
            "ContainerScreenController.onContainerSlotHovered",
            "ContainerScreenController.onContainerSlotUnhovered",
            "CreativeItemRegistry.mCreativeItems",
            "DBStorage.checkShutdownDone",
            "DBStorage.ctor",
            "DBStorage.mFullPath",
            "DBStorage.vtable",
            "DefaultDataLoadHelper.vtable",
            "Dimension.getBlockSourceFromMainChunkSource",
            "Dimension.getDimensionId",
            "Dimension.mSkyDarken",
            "FillingContainer.mItems",
            "FillingContainer.setItem",
            "FillingContainer.swapSlots",
            "Font.getLineLength",
            "FontHandle.dtor",
            "FontHandle.mDefaultFont",
            "FontHandle.size",
            "FontRepository.mLoadedFonts",
            "FrameRenderObject.mViews",
            "GameConnectionInfo.dtor",
            "GameConnectionInfo.mExternalServerName",
            "GameConnectionInfo.mUnresolvedUrl",
            "GameConnectionInfo.size",
            "GameControllerHandler_GameCore.refresh",
            "GameMode.attack",
            "GameMode.buildBlock",
            "GameMode.buildingTickDelay",
            "GameMode.continueDestroyBlock",
            "GameMode.getMaxPickRange",
            "GameMode.mPlayer",
            "GameMode.startDestroyBlock",
            "GameMode.useItem",
            "GameMode.useItemInteraction",
            "GameMode.vtable",
            "GlobalConstantBuffers.entityConstantBuffer",
            "GuiData.getGuiScale",
            "GuiMessage.ctor",
            "GuiMessage.ctorShape",
            "GuiMessage.ctorWithFlagWord",
            "GuiMessage.mAuthorXuid",
            "GuiMessage.mFilteredFullString",
            "GuiMessage.mFilteredFullStringEngaged",
            "GuiMessage.mFullString",
            "GuiMessage.mType",
            "GuiMessage.mUsername",
            "GuiMessage.size",
            "HIDController.resetKeys",
            "HIDControllerGameCoreDesktop.onKeyDown",
            "HIDControllerGameCoreDesktop.onKeyUp",
            "HIDControllerGameCoreDesktop.update",
            "HashedString.layout",
            "HudLocatorBarRenderer.drawIcon",
            "HudLocatorBarRenderer.render",
            "HudLocatorBarRenderer.vtable",
            "HudScreenController.handleSlotSelection",
            "HudScreenController.tick",
            "HudScreenController.vtable",
            "IMinecraftGame.grabMouse",
            "IMinecraftGame.releaseMouse",
            "IOptionRegistry.getHideHud",
            "IOptionRegistry.getUIProfile",
            "IOptionRegistry.getViewDistanceChunks",
            "IOptionRegistry.isGraphicsModeDeferred",
            "IOptionRegistry.setHideGUI",
            "ImageBuffer.ctorFromDescription",
            "ImageBuffer.mData",
            "ImageBuffer.mDeleter",
            "ImageBuffer.mSize",
            "ImageBuffer.size",
            "InputHandler.mAnyInputHandler",
            "IntOption.set",
            "Item.appendFormattedHovertext",
            "Item.getAnimationFrameFor",
            "Item.getMaxDamage",
            "Item.isGlint",
            "Item.mFullName",
            "Item.useOn",
            "ItemInHandRenderer.mMatBlendBlock",
            "ItemRegistryRef.getCreativeItemRegistry",
            "ItemRenderer.renderGuiItemNew",
            "ItemStack.copyCtor",
            "ItemStack.emptyCtor",
            "ItemStack.fromTag",
            "ItemStack.mNetIdVariant",
            "ItemStack.reinit",
            "ItemStack.size",
            "ItemStack.vtable",
            "ItemStackBase.dtor",
            "ItemStackBase.getDamageValue",
            "ItemStackBase.getMaxStackSize",
            "ItemStackBase.mBlock",
            "ItemStackBase.mCount",
            "ItemStackBase.mItem",
            "JsonValue.toStyledString",
            "KeyboardEventProcessor.feedKeyState",
            "LegacyClientNetworkHandler.vtable",
            "Level.getCameraPresets",
            "Level.getHitResult",
            "Level.getItemRegistry",
            "Level.getLevelSeed64",
            "Level.isClientSide",
            "LevelChunk.getBiome",
            "LevelChunk.getBlock",
            "LevelChunk.mSubChunks",
            "LevelData.mCommandsEnabled",
            "LevelData.mLevelName",
            "LevelRenderer.mAtlasTexture",
            "LevelRenderer.mLevelRendererPlayer",
            "LevelRenderer.mRenderChunkCoordinators",
            "LevelRenderer.renderLevel",
            "LevelRendererCamera.mActorRenderQueue",
            "LevelRendererCamera.mCameraPos",
            "LevelRendererCamera.mCameraTargetPos",
            "LevelRendererCamera.mWireframeMaterial",
            "LevelRendererCamera.preRenderUpdate",
            "LevelRendererCamera.renderBlockEntitiesBody",
            "LevelRendererPlayer.createViewRenderObject",
            "LevelRendererPlayer.getFov",
            "LevelRendererPlayer.onWillChangeDimension",
            "LevelRendererPlayer.queueRenderEntities",
            "LevelRendererPlayer.setupCamera",
            "LevelRendererPlayer.vtable",
            "LightPropagation.fillSubChunkGrid",
            "LightPropagation.fillSubChunkGridNoMode",
            "LightPropagation.tlsEmitterVector",
            "LightPropagationCoordinator.propagateForMesh",
            "LocalPlayer.applyTurnDelta",
            "LocalPlayer.mClientLocatorBar",
            "LocalWorldStarter.startLocalWorld",
            "MapPolicy.get",
            "MatrixStack.push",
            "MatrixStackRef.dtor",
            "Mesh.dtor",
            "Mesh.isValid",
            "Mesh.renderMeshTextureList",
            "Mesh.size",
            "MeshData.clear",
            "MeshData.dtor",
            "MeshData.moveAssign",
            "MinecraftGame.AppPlatformListener.vtable",
            "MinecraftGame.appPlatformListenerOffset",
            "MinecraftGame.applyActiveResourcePacks",
            "MinecraftGame.endFrame",
            "MinecraftGame.getWorldSystem",
            "MinecraftGame.grabMouse",
            "MinecraftGame.isInGame",
            "MinecraftGame.isPrimaryClientInstanceReady",
            "MinecraftGame.joinMultiplayerWithAddress",
            "MinecraftGame.mRealTimer",
            "MinecraftGame.mSimTimer",
            "MinecraftGame.onAppFocusGained",
            "MinecraftGame.onAppFocusLost",
            "MinecraftGame.update",
            "MinecraftGame.vtable",
            "MinecraftInputHandler.mInputHandler",
            "MinecraftPackets.createPacket",
            "MinecraftScreenModel.getVersionString",
            "MinecraftUIRenderContext.drawImage",
            "MinecraftUIRenderContext.drawRectangle",
            "MinecraftUIRenderContext.drawText",
            "MinecraftUIRenderContext.fillRectangle",
            "MinecraftUIRenderContext.flushImages",
            "MinecraftUIRenderContext.flushText",
            "MinecraftUIRenderContext.getTexture",
            "MinecraftUIRenderContext.mQueuedImages",
            "MinecraftUIRenderContext.mScreenContext",
            "MinecraftUIRenderContext.vtable",
            "MouseDevice.feed",
            "MouseDevice.feedWithControllerId",
            "MouseDevice.instance",
            "MoveInputState.bitDown",
            "MoveInputState.bitLeft",
            "MoveInputState.bitRight",
            "MoveInputState.bitSprintDown",
            "MoveInputState.bitUp",
            "MoveInputState.mAnalogMoveVector",
            "MoveInputState.maskJumpDown",
            "MoveInputState.maskSneakTest",
            "NbtIo.read",
            "NetworkConnection.mPeer",
            "NetworkConnection.mType",
            "NetworkPeer.receivePacket",
            "NetworkSystem.isServer",
            "NetworkSystem.mConnections",
            "NetworkSystem.send",
            "NetworkSystem.sendInternal",
            "NetworkWorldJoiner.mServerWorldJoiner",
            "OptionRegistry.getEffectiveMaxFramerateOption",
            "OptionRegistry.getIfValid",
            "OreUIRouter.pushRoute",
            "OreUIRouter.replaceRoute",
            "Packet.read",
            "PacketSender.send",
            "PacketTraceNetworkPeer.receivePacket",
            "ParticleSystemEngine.mLastTickTime",
            "ParticleSystemEngine.mParticleCount",
            "ParticleSystemEngine.mValid",
            "ParticleSystemEngine.tick",
            "Player.getAbilitiesComponent",
            "Player.getPlayerGameType",
            "Player.getPlayerPermissionLevel",
            "Player.isCreative",
            "Player.mGameMode",
            "Player.playerInventory",
            "PlayerInventory.mInventory",
            "PlayerInventory.mSelected",
            "PlayerListAddEntry.mId",
            "PlayerListAddEntry.mName",
            "PlayerListAddEntry.mSkin",
            "PlayerListAddEntry.mUUID",
            "PlayerListAddEntry.mXUID",
            "PlayerListAddEntry.size",
            "PlayerListPacket.mEntries",
            "PlayerListPacket.variantIndex",
            "PlayerSkinPacket.mSkin",
            "PlayerSkinPacket.mUUID",
            "PredictedMovementSystem.tickPredictedMovement",
            "PrimitiveShapeDataPayload.size",
            "PrimitiveShapesPacket.mShapes",
            "RawMoveInputComponent.mInputState",
            "RawMoveInputComponent.mRawMove",
            "ReadOnlyBinaryStream.dtor",
            "ReadOnlyBinaryStream.vtable",
            "RenderChunkCoordinator._setDirty",
            "RenderChunkCoordinator.vtable",
            "RenderMaterial.mBlendDestination",
            "RenderMaterial.mBlendSource",
            "RenderMaterial.mDepthWriteMask",
            "RenderMaterial.mEnableBlend",
            "RenderMaterialInfo.mMaterial",
            "RenderMetadata.engaged",
            "ResourceLocation.ctor",
            "ResourceLocation.size",
            "ScreenContext.camera",
            "ScreenContext.constantBuffers",
            "ScreenContext.currentShaderColor",
            "ScreenContext.guiData",
            "ScreenContext.meshContext",
            "ScreenContext.tessellator",
            "ScreenController.dtor",
            "ScreenController.isContainerScreen",
            "ScreenView.handleButtonEvent",
            "ScreenView.mController",
            "ScreenView.mInputMode",
            "ScreenView.mPointerButtonIds",
            "ScreenView.mPointerLocationPrevious",
            "ScreenView.mSize",
            "ScreenView.mVisualTree",
            "ScreenView.render",
            "SerializedSkinImpl.mDefaultGeometryName",
            "SerializedSkinImpl.mGeometryData",
            "SerializedSkinImpl.mIsPersona",
            "SerializedSkinImpl.mIsPremium",
            "SerializedSkinImpl.mResourcePatch",
            "SerializedSkinImpl.mSkinAnimatedImages",
            "SerializedSkinImpl.mSkinImage",
            "ServerWorldJoiner.joinExternalServer",
            "SharedConstants.NetworkProtocolVersion",
            "ShulkerBoxBlockItem.appendFormattedHovertext",
            "SkullBlockRenderer.renderSkull",
            "SkullBlockRenderer.renderSkullUpright",
            "StateVectorComponent.mPos",
            "StringByteInput.vtable",
            "SubChunk.isPlaceHolderSubChunk",
            "SubChunk.size",
            "SurvivalMode.vtable",
            "SystemUniforms.mNow",
            "TessellationPolicy.get",
            "Tessellator.begin",
            "Tessellator.clear",
            "Tessellator.end",
            "Tessellator.mTessellating",
            "TextureGroup.baseOffset",
            "TextureGroup.getTexture",
            "TextureGroup.textureGroupBase",
            "TextureGroup.unloadTexture",
            "TextureGroup.uploadTexture",
            "Timer.advanceTime",
            "UIControl._setCachedPosition",
            "UIControl.mCachedPosition",
            "UIControl.mCachedPositionDirty",
            "UIControl.mCachedPositionDirtyMask",
            "UIControl.mChildren",
            "UIControl.mIsVisibleInTree",
            "UIControl.mIsVisibleInTreeMask",
            "UIControl.mName",
            "UIControl.mSize",
            "UIScene.handleDirection",
            "UIScene.mScreenView",
            "UIScene.vtable",
            "VanillaBlockActorFactory.createBlockActor",
            "VanillaBlockActorFactory.createBlockActorWithFlag",
            "VanillaBlockStateTransformUtils.hasTransformableConnectionStates",
            "VanillaBlockStateTransformUtils.hasTransformableWallStates",
            "VanillaBlockStateTransformUtils.transformBlock",
            "VanillaBlockStateTransformUtils.transformConnectionStates",
            "VanillaBlockStateTransformUtils.transformWallConnectionTypes",
            "ViewRenderData.mCameraPos",
            "ViewRenderObject.mViewData",
            "ViewRenderObject.size",
            "VisualTree.mRootControl",
            "WorldSystem.mLocalWorldStarter",
            "WorldSystem.mNetworkWorldJoiner",
            "core.operator_delete",
            "core.operator_new",
            "core.tls_index",
        ]
    );
}

#[test]
fn entries_resolve_from_derive_then_from_the_cache_and_again_from_derive_once_the_cache_is_gone() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    let wanted = [id("Probe.run")];
    let rva = Some(u64::from(Probe::rva()));

    let first = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(first.source_of(&id("Probe.run")), Some("derive"));
    assert_eq!(first.get(&id("Probe.run")).map(|resolved| resolved.value), rva);

    let cached = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(cached.source_of(&id("Probe.run")), Some("cache"));
    assert_eq!(cached.get(&id("Probe.run")).map(|resolved| resolved.value), rva);

    std::fs::remove_dir_all(&paths.cache).unwrap();
    let derived = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(derived.source_of(&id("Probe.run")), Some("derive"));
    assert_eq!(derived.get(&id("Probe.run")).map(|resolved| resolved.value), rva);
}

#[test]
fn a_recipe_notes_change_resolves_from_the_cache() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    let wanted = [id("Probe.run")];
    let recipe = paths.recipes.join("probe.hcl");

    let first = StandaloneHost::resolve(&paths, &wanted);
    assert_eq!(first.source_of(&id("Probe.run")), Some("derive"));

    let text = std::fs::read_to_string(&recipe).unwrap();
    std::fs::write(
        &recipe,
        text.replacen(
            "kind = \"function\"",
            "kind = \"function\"
  notes = \"edited\"",
            1,
        ),
    )
    .unwrap();
    let second = StandaloneHost::resolve(&paths, &wanted);

    assert_eq!(second.source_of(&id("Probe.run")), Some("cache"));
}

#[test]
fn shipped_data_resolves_without_a_cache() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    let wanted = [id("Probe.run")];
    StandaloneHost::resolve(&paths, &wanted);
    std::fs::create_dir_all(&paths.data).unwrap();
    for item in std::fs::read_dir(&paths.cache).unwrap() {
        let item = item.unwrap();
        std::fs::copy(item.path(), paths.data.join(item.file_name())).unwrap();
    }
    std::fs::remove_dir_all(&paths.cache).unwrap();

    let table = StandaloneHost::resolve(&paths, &wanted);

    assert_eq!(table.source_of(&id("Probe.run")), Some("cache"));
    assert!(!paths.cache.exists() || std::fs::read_dir(&paths.cache).unwrap().next().is_none());
}

#[test]
fn an_unreadable_exe_leaves_every_entry_missing() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    std::fs::write(&paths.exe, b"not a pe file").unwrap();

    let table = StandaloneHost::resolve(&paths, &[id("Probe.run")]);

    assert_eq!(table.missing(), vec![id("Probe.run")]);
}

#[test]
fn start_logs_coverage_defers_the_heap_check_and_works_once() {
    let root = tempfile::tempdir().unwrap();
    let paths = Probe::standalone_layout(root.path());
    std::fs::create_dir_all(&paths.logs).unwrap();
    LoaderLog::install(&paths.logs).unwrap();

    StandaloneHost::start(&paths, 0x1000).unwrap();

    assert!(LoaderState::get().is_some());
    assert!(paths.mods.is_dir());
    let log = LogFile::wait_for(&paths.logs.join("loader.log"), "deferred");
    assert!(log.contains("resolved 0/470"), "{log}");
    assert!(log.contains("game heap round trip deferred"), "{log}");
    assert!(!log.contains("game heap ok"), "{log}");
    assert!(StandaloneHost::start(&paths, 0x1000).is_err());
}

struct Instance;

impl Instance {
    fn layout_with_log(root: &std::path::Path) -> amml::HostPaths {
        let paths = Probe::standalone_layout(root);
        std::fs::create_dir_all(&paths.logs).unwrap();
        LoaderLog::install(&paths.logs).unwrap();
        paths
    }

    fn write(paths: &amml::HostPaths, text: &str) {
        std::fs::create_dir_all(&paths.loader).unwrap();
        std::fs::write(&paths.instance, text).unwrap();
    }

    fn log_after_marker(paths: &amml::HostPaths) -> String {
        curia::info!("marker for the test");
        LogFile::wait_for(&paths.logs.join("loader.log"), "marker for the test")
    }
}

#[test]
fn install_redirect_without_instance_file_is_none() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_none());
    let log = Instance::log_after_marker(&paths);
    assert!(!log.contains("instance.hcl"), "{log}");
}

#[test]
fn install_redirect_with_bad_file_is_none_and_logged() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());
    Instance::write(&paths, "data_dir = 3");

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_none());
    let log = Instance::log_after_marker(&paths);
    let lines: Vec<&str> = log.lines().filter(|line| line.contains("instance.hcl")).collect();
    assert_eq!(lines.len(), 1, "{log}");
    assert!(lines[0].contains("ERROR"), "{log}");
}

#[test]
fn install_redirect_with_data_dir_redirects_this_process_and_creates_the_folder() {
    let root = tempfile::tempdir().unwrap();
    let paths = Instance::layout_with_log(root.path());
    let data_dir = root.path().join("data");
    let escaped = data_dir.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
    Instance::write(&paths, &format!("data_dir = \"{escaped}\"\nfuture_key = true\n"));

    let redirect = StandaloneHost::install_redirect(&paths);

    assert!(redirect.is_some_and(|redirect| !redirect.slots().is_empty()));
    assert!(data_dir.is_dir());
    assert_eq!(std::env::var("APPDATA").unwrap(), data_dir.to_string_lossy());
    let log = Instance::log_after_marker(&paths);
    assert!(log.contains("data folder redirected"), "{log}");
    assert!(log.contains("future_key"), "{log}");
}

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
            "Actor.aabbShapeComponent",
            "Actor.actorRotationComponent",
            "Actor.getDimension",
            "Actor.getItemInHandSlot",
            "Actor.getRuntimeID",
            "Actor.isAlive",
            "Actor.stateVectorComponent",
            "ActorRotationComponent.mRot",
            "AnimatedImageData.mImage",
            "AnimatedImageData.mType",
            "AnimatedImageData.size",
            "BaseActorRenderContext.ctor",
            "BaseActorRenderContext.dtor",
            "BaseActorRenderContext.size",
            "BitmapFont._getCharWidth",
            "BitmapFont.vtable",
            "Block.mDirectData.mLightEmission",
            "BlockSource.getLightColor",
            "BlockTessellatorCache.getLightColor",
            "ClientEntityWaypoint.mActorId",
            "ClientEntityWaypoint.vtable",
            "ClientExitLevel.hook",
            "ClientInputUpdateSystem.extractRawHIDInput",
            "ClientInstance.gameCallbacksOffset",
            "ClientInstance.getCurrentUIRouter",
            "ClientInstance.getFontHandle",
            "ClientInstance.getLocalPlayer",
            "ClientInstance.getMinecraftGame",
            "ClientInstance.getMinecraftInput",
            "ClientInstance.getOptions",
            "ClientInstance.getPacketSender",
            "ClientInstance.getTextureGroup",
            "ClientInstance.getTopScreenName",
            "ClientInstance.isShowingMenu",
            "ClientInstance.isWorldActive",
            "ClientInstance.onStartJoinGame",
            "ClientInstance.onTick",
            "ClientInstance.requestLeaveGame",
            "ClientInstance.update",
            "ClientInstance.vtable",
            "ClientLocatorBar.listHead",
            "ClientLocatorBar.nodeWaypoint",
            "ClientNetworkHandler.handle.ContainerClosePacket",
            "ClientNetworkHandler.handle.ContainerOpenPacket",
            "ClientNetworkHandler.handle.ItemStackResponsePacket",
            "ClientNetworkHandler.handle.PlayerListPacket",
            "ClientNetworkHandler.handle.PlayerSkinPacket",
            "ClientWaypoint.mColor",
            "ClientWaypoint.mColorEngaged",
            "ClientWaypoint.mVisible",
            "CommandRequestPacket.mCommand",
            "CommandRequestPacket.size",
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
            "Dimension.getDimensionId",
            "FillingContainer.mItems",
            "FillingContainer.swapSlots",
            "Font.getLineLength",
            "FontHandle.dtor",
            "FontHandle.mDefaultFont",
            "FontHandle.size",
            "FontRepository.mLoadedFonts",
            "GameControllerHandler_GameCore.refresh",
            "GameMode.attack",
            "GameMode.continueDestroyBlock",
            "GameMode.mPlayer",
            "GameMode.startDestroyBlock",
            "GameMode.useItem",
            "GameMode.useItemInteraction",
            "GameMode.vtable",
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
            "IOptionRegistry.setHideGUI",
            "ImageBuffer.ctorFromDescription",
            "ImageBuffer.mData",
            "ImageBuffer.mDeleter",
            "ImageBuffer.mSize",
            "ImageBuffer.size",
            "InputHandler.mAnyInputHandler",
            "Item.appendFormattedHovertext",
            "Item.getAnimationFrameFor",
            "Item.getMaxDamage",
            "Item.isGlint",
            "Item.mFullName",
            "Item.useOn",
            "ItemRenderer.renderGuiItemNew",
            "ItemStack.fromTag",
            "ItemStack.mNetIdVariant",
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
            "LevelRenderer.mRenderChunkCoordinators",
            "LevelRenderer.renderLevel",
            "LevelRendererPlayer.getFov",
            "LevelRendererPlayer.onWillChangeDimension",
            "LightPropagation.fillSubChunkGrid",
            "LightPropagation.fillSubChunkGridNoMode",
            "LightPropagation.tlsEmitterVector",
            "LightPropagationCoordinator.propagateForMesh",
            "LocalPlayer.applyTurnDelta",
            "LocalPlayer.mClientLocatorBar",
            "LocalWorldStarter.startLocalWorld",
            "MinecraftGame.AppPlatformListener.vtable",
            "MinecraftGame.appPlatformListenerOffset",
            "MinecraftGame.applyActiveResourcePacks",
            "MinecraftGame.getWorldSystem",
            "MinecraftGame.grabMouse",
            "MinecraftGame.isPrimaryClientInstanceReady",
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
            "NetworkWorldJoiner.mServerWorldJoiner",
            "OptionRegistry.getIfValid",
            "OreUIRouter.pushRoute",
            "OreUIRouter.replaceRoute",
            "Packet.read",
            "PacketSender.send",
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
            "RawMoveInputComponent.mInputState",
            "RawMoveInputComponent.mRawMove",
            "ReadOnlyBinaryStream.dtor",
            "ReadOnlyBinaryStream.vtable",
            "RenderChunkCoordinator._setDirty",
            "RenderChunkCoordinator.vtable",
            "ResourceLocation.ctor",
            "ResourceLocation.size",
            "ScreenContext.guiData",
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
            "ShulkerBoxBlockItem.appendFormattedHovertext",
            "StateVectorComponent.mPos",
            "SurvivalMode.vtable",
            "TextureGroup.baseOffset",
            "TextureGroup.getTexture",
            "TextureGroup.textureGroupBase",
            "TextureGroup.unloadTexture",
            "TextureGroup.uploadTexture",
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
    assert!(log.contains("resolved 0/250"), "{log}");
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

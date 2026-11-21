use crate::structs::{AdapterNeed, AdapterShape, Availability, NeedKind};

static FRAME_UPDATE: [AdapterShape; 1] =
    AdapterShape::single("default", &["MinecraftGame.update", AdapterCatalog::MINECRAFT_GAME_VTABLE]);
static CLIENT_TICK: [AdapterShape; 1] = AdapterShape::single("default", &["ClientInstance.update"]);
// Version 2 raises only the ticks of the primary client: the client is the GameCallbacks subobject minus the offset.
static CLIENT_GAME_TICK: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.onTick", "ClientInstance.gameCallbacksOffset"]);
static JOIN_STARTED: [AdapterShape; 1] =
    AdapterShape::single("default", &["ClientInstance.onStartJoinGame", AdapterCatalog::CLIENT_INSTANCE_VTABLE]);
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
static DIMENSION_CHANGING: [AdapterShape; 1] =
    AdapterShape::single("default", &["LevelRendererPlayer.onWillChangeDimension"]);
// The game's any-input handler: getMinecraftInput()->mInputHandler->mAnyInputHandler.
static CLIENT_ACTIVITY: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &["ClientInstance.getMinecraftInput", "MinecraftInputHandler.mInputHandler", "InputHandler.mAnyInputHandler"],
);
static IDLE_SCREEN_OPEN: [AdapterShape; 1] =
    AdapterShape::single("default", &["OreUIRouter.pushRoute", "OreUIRouter.replaceRoute"]);

// The local player of the primary client, read through its Actor vtable. Later appends to the actor table raise the
// row's version (`AdapterNeed::area_at`).
// Version 2 adds the position, eye position, rotation and dimension reads.
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
    ],
);

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
static UI_TEXTURES: [AdapterShape; 1] = AdapterShape::single(
    "default",
    &[
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

static NEEDS: [AdapterNeed; 95] = [
    AdapterNeed::event("frame.update", &FRAME_UPDATE),
    AdapterNeed::event("client.tick", &CLIENT_TICK),
    AdapterNeed::event_at("client.game_tick", 2, &CLIENT_GAME_TICK),
    AdapterNeed::event("world.join_started", &JOIN_STARTED),
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
    AdapterNeed::area("client.leave_world", &CLIENT_LEAVE_WORLD),
    AdapterNeed::area("client.can_join", &CLIENT_CAN_JOIN),
    AdapterNeed::area("client.start_local_world", &CLIENT_START_LOCAL_WORLD),
    AdapterNeed::area("client.join_server", &CLIENT_JOIN_SERVER),
    AdapterNeed::event("client.join_server_result", &CLIENT_JOIN_SERVER),
    AdapterNeed::area("client.open_menu", &CLIENT_OPEN_MENU),
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
    AdapterNeed::area_at("actor.read", 2, &ACTOR_READ),
    AdapterNeed::area_at("inventory.read", 4, &INVENTORY_READ),
    AdapterNeed::area("inventory.request", &INVENTORY_REQUEST),
    AdapterNeed::event("inventory.request_done", &INVENTORY_REQUEST),
    AdapterNeed::area("lighting.terrain", &LIGHTING_TERRAIN),
    AdapterNeed::area("lighting.entity", &LIGHTING_ENTITY),
    AdapterNeed::area("lighting.vibrant", &LIGHTING_VIBRANT),
    AdapterNeed::event("player.block_break_start", &BLOCK_BREAK_START),
    AdapterNeed::event("player.block_break_continue", &BLOCK_BREAK_CONTINUE),
    AdapterNeed::event("player.item_use", &ITEM_USE),
    AdapterNeed::event("player.attack", &PLAYER_ATTACK),
    AdapterNeed::event("player.item_use_on_block", &ITEM_USE_ON_BLOCK),
    AdapterNeed::area("client.activity", &CLIENT_ACTIVITY),
    AdapterNeed::event("client.idle_screen_open", &IDLE_SCREEN_OPEN),
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

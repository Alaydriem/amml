use amml_data::{AdapterCatalog, Availability, NeedKind};

fn availability_of(resolved: &[&str], event: &str) -> Availability {
    AdapterCatalog::availability(&|name| resolved.contains(&name)).into_iter().find(|row| row.id == event).unwrap()
}

#[test]
fn an_event_whose_entries_all_resolve_is_available() {
    let row = availability_of(&["ClientInstance.update"], "client.tick");
    assert_eq!(row.shape, Some("default"));
    assert!(row.missing.is_empty());
}

#[test]
fn a_missing_entry_is_named() {
    let row = availability_of(&[], "client.tick");
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, vec!["ClientInstance.update"]);
}

#[test]
fn loader_events_are_always_available() {
    assert_eq!(availability_of(&[], "mods.loaded").shape, Some("loader"));
}

#[test]
fn every_catalog_entry_has_a_recipe() {
    let recipes = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../recipes");
    let set = amml_data::RecipeLoader::load_dir(&recipes).unwrap();
    for need in AdapterCatalog::needs() {
        for shape in need.shapes {
            for entry in shape.entries.iter().chain(shape.optional) {
                assert!(set.get(&amml_data::EntryId((*entry).to_string())).is_some(), "{} needs {entry}", need.id);
            }
        }
    }
}

#[test]
fn the_key_event_takes_the_hid_shape_when_its_entries_resolve() {
    let hid = [
        "HIDControllerGameCoreDesktop.onKeyDown",
        "HIDControllerGameCoreDesktop.onKeyUp",
        "HIDControllerGameCoreDesktop.update",
    ];
    assert_eq!(availability_of(&hid, "input.key").shape, Some("hid"));
}

#[test]
fn the_hid_shape_names_the_key_reset_as_optional_and_holds_without_it() {
    let hid = [
        "HIDControllerGameCoreDesktop.onKeyDown",
        "HIDControllerGameCoreDesktop.onKeyUp",
        "HIDControllerGameCoreDesktop.update",
    ];
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "input.key").unwrap();
    let shape = need.shapes.iter().find(|shape| shape.name == "hid").unwrap();

    assert_eq!(shape.optional, ["HIDController.resetKeys"]);
    assert!(!shape.entries.contains(&"HIDController.resetKeys"));
    let row = availability_of(&hid, "input.key");
    assert_eq!(row.shape, Some("hid"));
    assert!(row.missing.is_empty());
}

#[test]
fn the_key_event_takes_the_key_processor_shape_on_70x() {
    let row = availability_of(&["KeyboardEventProcessor.feedKeyState"], "input.key");
    assert_eq!(row.shape, Some("key_processor"));
    assert!(row.missing.is_empty());
}

#[test]
fn the_mouse_event_names_the_first_shape_when_neither_resolves() {
    let row = availability_of(&[], "input.mouse");
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, vec!["MouseDevice.feed", "MouseDevice.instance"]);
}

#[test]
fn area_needs_are_marked_area() {
    assert_eq!(availability_of(&[], "input.key_inject").kind, NeedKind::Area);
    assert_eq!(availability_of(&[], "input.key").kind, NeedKind::Event);
}

#[test]
fn window_needs_take_the_window_shape_and_need_no_entry() {
    for id in
        ["window.read", "window.request_size", "window.cursor_clip", "window.pointer", "window.size", "window.cursor"]
    {
        let row = availability_of(&[], id);
        assert_eq!(row.shape, Some(AdapterCatalog::WINDOW_SHAPE), "{id}");
        assert!(row.missing.is_empty(), "{id}");
    }
}

#[test]
fn overlay_needs_take_the_overlay_shape_and_need_no_entry() {
    for id in [
        "overlay.surface",
        "overlay.frame_present",
        "overlay.frame_presented",
        "overlay.swap_chain_resized",
        "overlay.surface_lost",
        "overlay.render_device_reset",
        "overlay.screenshot",
        "overlay.capture",
        "overlay.screenshot_taken",
        "overlay.screenshot_written",
    ] {
        let row = availability_of(&[], id);
        assert_eq!(row.shape, Some(AdapterCatalog::OVERLAY_SHAPE), "{id}");
        assert!(row.missing.is_empty(), "{id}");
    }
    assert_eq!(availability_of(&[], "overlay.surface").kind, NeedKind::Area);
    assert_eq!(availability_of(&[], "overlay.screenshot").kind, NeedKind::Area);
    assert_eq!(availability_of(&[], "overlay.capture").kind, NeedKind::Area);
}

#[test]
fn camera_and_dimension_rows_name_their_entries() {
    let mut resolved: Vec<&str> =
        AdapterCatalog::needs().iter().flat_map(|need| need.shapes).flat_map(|shape| shape.entries).copied().collect();
    resolved.push("LevelRendererPlayer.getFov");
    resolved.retain(|name| *name != "LevelRendererPlayer.onWillChangeDimension");

    assert_eq!(availability_of(&resolved, "camera.fov").shape, Some("default"));
    let dimension = availability_of(&resolved, "world.dimension_changing");
    assert_eq!(dimension.shape, None);
    assert_eq!(dimension.missing, vec!["LevelRendererPlayer.onWillChangeDimension"]);
}

const ACTOR_V2: [&str; 9] = [
    "Actor.isAlive",
    "ClientInstance.getLocalPlayer",
    "Actor.stateVectorComponent",
    "StateVectorComponent.mPos",
    "Actor.aabbShapeComponent",
    "Actor.actorRotationComponent",
    "ActorRotationComponent.mRot",
    "Actor.getDimension",
    "Dimension.getDimensionId",
];

#[test]
fn the_actor_read_area_needs_the_alive_slot_the_local_player_and_the_v2_reads() {
    let mut all = ACTOR_V2.to_vec();
    all.extend(["Actor.getAttachPos", "Dimension.getBlockSourceFromMainChunkSource", "Actor.getViewVector"]);
    let row = availability_of(&all, "actor.read");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 4));
    assert_eq!(availability_of(&all[1..], "actor.read").missing, ["Actor.isAlive"]);
    assert_eq!(availability_of(&all[..2], "actor.read").missing.len(), 10);
}

#[test]
fn actor_read_v3_needs_the_block_source_entries() {
    let v2_only = availability_of(&ACTOR_V2, "actor.read");
    assert_eq!(v2_only.shape, None);
    assert_eq!(
        v2_only.missing,
        ["Actor.getAttachPos", "Dimension.getBlockSourceFromMainChunkSource", "Actor.getViewVector"]
    );
}

#[test]
fn actor_read_v4_needs_the_view_vector_call() {
    let mut v3 = ACTOR_V2.to_vec();
    v3.extend(["Actor.getAttachPos", "Dimension.getBlockSourceFromMainChunkSource"]);
    let row = availability_of(&v3, "actor.read");
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, ["Actor.getViewVector"]);
}

#[test]
fn the_game_type_row_needs_the_getter() {
    let row = availability_of(&["Player.getPlayerGameType"], "actor.game_type");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    assert_eq!(availability_of(&[], "actor.game_type").missing, ["Player.getPlayerGameType"]);
}

#[test]
fn the_permission_row_has_one_shape_per_build() {
    let getter = availability_of(&["Player.getPlayerPermissionLevel"], "actor.permission");
    assert_eq!(getter.shape, Some(AdapterCatalog::PERMISSION_GETTER_SHAPE));
    let abilities =
        availability_of(&["Player.getAbilitiesComponent", "AbilitiesComponent.mPlayerPermissions"], "actor.permission");
    assert_eq!(abilities.shape, Some(AdapterCatalog::PERMISSION_ABILITIES_SHAPE));
    // The abilities shape needs the byte offset as well as the getter.
    assert_eq!(availability_of(&["Player.getAbilitiesComponent"], "actor.permission").shape, None);
    let neither = availability_of(&[], "actor.permission");
    assert_eq!(neither.shape, None);
    assert_eq!(neither.missing, ["Player.getPlayerPermissionLevel"]);
}

#[test]
fn look_write_needs_the_rotation_entries() {
    let all = ["Actor.actorRotationComponent", "ActorRotationComponent.mRot", "Actor.setYHeadRotations"];
    let row = availability_of(&all, "actor.look_write");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    for entry in all {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "actor.look_write").missing, vec![entry], "{entry}");
    }
}

#[test]
fn actor_health_needs_the_attribute_read_and_the_entity_context() {
    let row = availability_of(&["ActorAttribute.getHealth", "Actor.mEntityContext"], "actor.health");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    assert_eq!(availability_of(&["ActorAttribute.getHealth"], "actor.health").missing, ["Actor.mEntityContext"]);
}

#[test]
fn sky_darken_row_needs_its_field() {
    let row = availability_of(&["Dimension.mSkyDarken", "Actor.getDimension"], "world.sky_darken");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    assert_eq!(availability_of(&["Actor.getDimension"], "world.sky_darken").missing, ["Dimension.mSkyDarken"]);
}

#[test]
fn chunk_read_row_needs_the_chunk_entries_and_the_block_source_chain() {
    let all = entries_of("world.chunk_read");
    assert!(all.contains(&"Dimension.getBlockSourceFromMainChunkSource"));
    assert!(all.contains(&"Actor.getDimension"));
    let row = AdapterCatalog::availability(&|name| all.contains(&name))
        .into_iter()
        .find(|row| row.id == "world.chunk_read")
        .unwrap();
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    let without = AdapterCatalog::availability(&|name| name != "Block.mBlockType" && all.contains(&name))
        .into_iter()
        .find(|row| row.id == "world.chunk_read")
        .unwrap();
    assert_eq!(without.missing, ["Block.mBlockType"]);
}

// The `world.actors_in` row at `version` for the entries `resolved`.
fn actors_in_row(resolved: &[&str], version: u32) -> Availability {
    AdapterCatalog::availability(&|name| resolved.contains(&name))
        .into_iter()
        .find(|row| row.id == "world.actors_in" && row.version == version)
        .unwrap()
}

#[test]
fn actors_in_v2_needs_the_actor_type_entry_and_v1_does_not() {
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "world.actors_in" && need.version == 2).unwrap();
    let v2 = need.shapes[0].entries;
    let without_type: Vec<&str> = v2.iter().copied().filter(|name| *name != "ActorTypeComponent.typeId").collect();

    let v1_without = actors_in_row(&without_type, 1);
    let v2_without = actors_in_row(&without_type, 2);
    let (v1_with, v2_with) = (actors_in_row(v2, 1), actors_in_row(v2, 2));

    assert_eq!(v1_without.shape, Some("default"));
    assert_eq!((v2_without.shape, v2_without.missing), (None, vec!["ActorTypeComponent.typeId"]));
    assert_eq!((v1_with.shape, v2_with.shape), (Some("default"), Some("default")));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "world.actors_in"), Some(2));
}

#[test]
fn map_color_row_has_one_shape_per_build_and_biomes_need_only_what_they_use() {
    let component = entries_of("world.map_color");
    let on_26_63 = AdapterCatalog::availability(&|name| name != "Block.getMapColor" && component.contains(&name))
        .into_iter()
        .find(|row| row.id == "world.map_color")
        .unwrap();
    let on_70_26 = AdapterCatalog::availability(&|name| {
        name != "Block.tryGetComponent.BlockMapColorComponent" && component.contains(&name)
    })
    .into_iter()
    .find(|row| row.id == "world.map_color")
    .unwrap();
    assert_eq!((on_26_63.shape, on_70_26.shape), (Some("component"), Some("block")));
    let biomes = entries_of("world.biomes");
    assert!(biomes.contains(&"HashedString.layout"));
    for unused in ["Level.getBiomeRegistry", "BiomeRegistry.lookupById", "ClientInstance.getLevel"] {
        assert!(!biomes.contains(&unused), "{unused}");
    }
    for shape in AdapterCatalog::needs().iter().find(|need| need.id == "world.map_color").unwrap().shapes {
        assert!(shape.entries.contains(&"Block.mBlockType"), "{}", shape.name);
    }
}

#[test]
fn scan_surfaces_has_one_shape_per_build_and_needs_the_chunk_reads() {
    let all = entries_of("world.scan_surfaces");
    let row = |without: &str| {
        AdapterCatalog::availability(&|name| name != without && all.contains(&name))
            .into_iter()
            .find(|row| row.id == "world.scan_surfaces")
            .unwrap()
    };
    assert_eq!(row("Block.canProvideFullSupportWithFlag").shape, Some("support"));
    assert_eq!(row("Block.canProvideFullSupport").shape, Some("support_flag"));
    assert_eq!(row("LevelChunk.getBlock").shape, None);
    for needed in ["BlockType.getCollisionShape", "BlockSource.getBrightnessPair", "Block.mBlockType"] {
        assert_eq!(row(needed).shape, None, "{needed}");
    }
}

#[test]
fn block_changed_needs_the_listener_the_source_and_the_type_reads() {
    let all = entries_of("world.block_changed");
    let row = |without: &str| {
        AdapterCatalog::availability(&|name| name != without && all.contains(&name))
            .into_iter()
            .find(|row| row.id == "world.block_changed")
            .unwrap()
    };
    assert_eq!(availability_of(&all, "world.block_changed").shape, Some("default"));
    for needed in [
        "BlockSourceListener.vtable",
        "BlockSourceListener.onSourceDestroyed",
        "BlockSourceListener.onBlockChanged",
        "BlockSource.addListener",
        "BlockSource.removeListener",
        "BlockSource.getDimensionId",
        "Block.mBlockType",
        "BlockType.mNameInfo.mFullName",
        "HashedString.layout",
        "Dimension.getBlockSourceFromMainChunkSource",
    ] {
        assert_eq!(row(needed).shape, None, "{needed}");
    }
}

#[test]
fn world_info_needs_the_getters_the_level_reads_and_the_storage_entries() {
    let all = entries_of("client.world_info");
    let row = |without: &str| {
        AdapterCatalog::availability(&|name| name != without && all.contains(&name))
            .into_iter()
            .find(|row| row.id == "client.world_info")
            .unwrap()
    };
    assert_eq!(availability_of(&all, "client.world_info").shape, Some("default"));
    for needed in [
        "ClientInstance.getGameConnectionInfo",
        "GameConnectionInfo.size",
        "GameConnectionInfo.dtor",
        "Level.getLevelSeed64",
        "LevelData.mLevelName",
        "DBStorage.ctor",
        "DBStorage.checkShutdownDone",
        "DBStorage.mFullPath",
    ] {
        assert_eq!(row(needed).shape, None, "{needed}");
    }
}

#[test]
fn served_version_gives_the_catalog_version_or_none() {
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "actor.read"), Some(4));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Event, "input.injected_released"), Some(2));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Event, "actor.read"), None);
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "no.such_area"), None);
}

fn entries_of(id: &str) -> Vec<&'static str> {
    let need = AdapterCatalog::needs().iter().find(|need| need.id == id).unwrap();
    need.shapes.iter().flat_map(|shape| shape.entries.iter().copied()).collect()
}

#[test]
fn ui_rows_need_their_entries() {
    for id in ["ui.screen_render", "ui.screen", "ui.screen_closing", "ui.screen_changed"] {
        let all = entries_of(id);
        assert!(!all.is_empty(), "{id}");
        assert!(availability_of(&all, id).shape.is_some(), "{id}");
    }
    let without_size: Vec<&str> =
        entries_of("ui.screen").into_iter().filter(|name| *name != "ScreenView.mSize").collect();
    let row = availability_of(&without_size, "ui.screen");
    assert_eq!(row.shape, None);
    assert_eq!(row.missing, vec!["ScreenView.mSize"]);
}

#[test]
fn the_ui_rows_name_the_hooks_and_reads_of_the_screen_lifecycle() {
    assert!(entries_of("ui.screen_render").contains(&"ScreenView.render"));
    assert!(entries_of("ui.screen").contains(&"ScreenView.mController"));
    assert_eq!(entries_of("ui.screen_closing"), ["ScreenController.dtor"]);
    assert!(entries_of("ui.screen_changed").contains(&"ClientInstance.getTopScreenName"));
}

#[test]
fn the_draw_and_texture_rows_need_their_entries_and_ask_for_the_queue_optionally() {
    for id in ["ui.draw", "ui.textures"] {
        let all = entries_of(id);
        assert!(availability_of(&all, id).shape.is_some(), "{id}");
    }
    let draw = entries_of("ui.draw");
    assert!(draw.contains(&"MinecraftUIRenderContext.drawImage") && draw.contains(&"GuiData.getGuiScale"));
    assert!(!draw.contains(&"MinecraftUIRenderContext.mQueuedImages"));
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "ui.draw").unwrap();
    assert_eq!(need.shapes[0].optional, ["MinecraftUIRenderContext.mQueuedImages"]);
    // The item draw is version 2 and its entries are required.
    assert_eq!(need.version, 2);
    for entry in [
        "ItemRenderer.renderGuiItemNew",
        "BaseActorRenderContext.ctor",
        "BaseActorRenderContext.dtor",
        "BaseActorRenderContext.size",
        "ClientInstance.getMinecraftGame",
        "Item.getAnimationFrameFor",
    ] {
        assert!(draw.contains(&entry), "{entry}");
        let without: Vec<&str> = draw.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "ui.draw").missing, vec![entry]);
    }
    let without_flush: Vec<&str> =
        draw.into_iter().filter(|name| *name != "MinecraftUIRenderContext.flushImages").collect();
    assert_eq!(availability_of(&without_flush, "ui.draw").missing, vec!["MinecraftUIRenderContext.flushImages"]);
    assert!(entries_of("ui.textures").contains(&"TextureGroup.uploadTexture"));
}

#[test]
fn the_container_calls_are_required_entries_of_ui_screen_version_two() {
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "ui.screen").unwrap();
    assert_eq!(need.version, 2);
    assert!(need.shapes[0].optional.is_empty());
    let all = entries_of("ui.screen");
    assert!(availability_of(&all, "ui.screen").shape.is_some());
    // The area is not offered at version 2 without a container entry.
    for entry in [
        "ContainerScreenController.getVisualItemStackImpl",
        "ContainerScreenController._handleAutoPlace",
        "ContainerManagerController.handleSwap",
    ] {
        assert!(all.contains(&entry), "{entry}");
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "ui.screen").missing, vec![entry]);
    }
}

#[test]
fn the_hover_events_need_their_handler_and_the_view_draw() {
    assert!(entries_of("ui.container_slot_hover").contains(&"ContainerScreenController.onContainerSlotHovered"));
    assert!(entries_of("ui.container_slot_unhover").contains(&"ContainerScreenController.onContainerSlotUnhovered"));
    for id in ["ui.container_slot_hover", "ui.container_slot_unhover"] {
        let all = entries_of(id);
        assert!(all.contains(&"ScreenView.render") && all.contains(&"ScreenView.mController"), "{id}");
        assert!(availability_of(&all, id).shape.is_some(), "{id}");
        let without_render: Vec<&str> = all.into_iter().filter(|name| *name != "ScreenView.render").collect();
        assert_eq!(availability_of(&without_render, id).missing, vec!["ScreenView.render"], "{id}");
    }
}

#[test]
fn inventory_read_is_served_at_version_four_and_the_contents_entries_are_required() {
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "inventory.read"), Some(4));
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "inventory.read").unwrap();
    let all = entries_of("inventory.read");
    assert!(availability_of(&all, "inventory.read").shape.is_some());
    for entry in ["ItemStack.fromTag", "ItemStackBase.dtor", "CompoundTag.nodeVariantIndex", "CompoundTag.vtable"] {
        assert!(need.shapes[0].entries.contains(&entry), "{entry}");
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "inventory.read").missing, vec![entry]);
    }
}

#[test]
fn the_text_row_needs_every_font_entry() {
    let all = entries_of("ui.text");
    assert!(availability_of(&all, "ui.text").shape.is_some());
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "ui.text").unwrap();
    assert_eq!((need.version, need.shapes[0].optional.len()), (1, 0));
    for entry in &all {
        let without: Vec<&str> = all.iter().copied().filter(|name| name != entry).collect();
        assert_eq!(availability_of(&without, "ui.text").missing, vec![*entry]);
    }
}

#[test]
fn the_ui_input_rows_need_their_handler_and_layout_entries() {
    let pointer =
        ["ScreenView.handleButtonEvent", "ScreenView.mPointerButtonIds", "ScreenView.mPointerLocationPrevious"];
    for entry in pointer {
        assert!(entries_of("input.ui_pointer").contains(&entry), "{entry}");
        let without: Vec<&str> = pointer.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "input.ui_pointer").missing, vec![entry]);
    }
    // The input mode is optional: the row holds without it.
    assert!(availability_of(&pointer, "input.ui_pointer").shape.is_some());
    assert!(!entries_of("input.ui_pointer").contains(&"ScreenView.mInputMode"));
    let optional =
        AdapterCatalog::needs().iter().find(|need| need.id == "input.ui_pointer").unwrap().shapes[0].optional;
    assert_eq!(optional, ["ScreenView.mInputMode"]);

    let scroll = ["UIScene.handleDirection", "UIScene.mScreenView", "UIScene.vtable"];
    for entry in scroll {
        assert!(entries_of("input.ui_scroll").contains(&entry), "{entry}");
        let without: Vec<&str> = scroll.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "input.ui_scroll").missing, vec![entry]);
    }

    let hovertext = ["ShulkerBoxBlockItem.appendFormattedHovertext", "Item.appendFormattedHovertext"];
    for entry in hovertext {
        assert!(entries_of("ui.item_hovertext").contains(&entry), "{entry}");
        let without: Vec<&str> = hovertext.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "ui.item_hovertext").missing, vec![entry]);
    }
}

#[test]
fn the_locator_bar_row_needs_the_draw_entries_and_the_waypoint_layout() {
    let entries = [
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
    ];
    for entry in entries {
        assert!(entries_of("ui.locator_bar_icon").contains(&entry), "{entry}");
        let without: Vec<&str> = entries.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "ui.locator_bar_icon").missing, vec![entry]);
    }
    assert!(availability_of(&entries, "ui.locator_bar_icon").shape.is_some());
}

#[test]
fn the_inventory_request_rows_need_the_packet_calls_and_the_container_entries() {
    let entries = [
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
    ];
    let pick_entries = [
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
    ];
    // The event stays on the packet entries; the area, at version 2, also needs the creative pick entries.
    for (id, wanted) in [
        ("inventory.request", [entries.as_slice(), pick_entries.as_slice()].concat()),
        ("inventory.request_done", entries.to_vec()),
    ] {
        for entry in &wanted {
            assert!(entries_of(id).contains(entry), "{id} {entry}");
            let without: Vec<&str> = wanted.iter().copied().filter(|name| name != entry).collect();
            assert_eq!(availability_of(&without, id).missing, vec![*entry], "{id}");
        }
        assert!(availability_of(&wanted, id).shape.is_some());
    }
    assert_eq!(AdapterCatalog::served_version(amml_data::NeedKind::Area, "inventory.request"), Some(2));
    assert_eq!(AdapterCatalog::served_version(amml_data::NeedKind::Event, "inventory.request_done"), Some(1));
}

#[test]
fn the_network_rows_need_their_packet_and_skin_entries() {
    let rows: [(&str, &[&str]); 6] = [
        (
            "network.player_list",
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
        ),
        (
            "network.player_skin",
            &[
                "LegacyClientNetworkHandler.vtable",
                "ClientNetworkHandler.handle.PlayerSkinPacket",
                "PlayerSkinPacket.mUUID",
                "PlayerSkinPacket.mSkin",
            ],
        ),
        ("network.skin", &["SerializedSkinImpl.mSkinImage"]),
        (
            "network.skin_animated",
            &[
                "SerializedSkinImpl.mSkinAnimatedImages",
                "AnimatedImageData.size",
                "AnimatedImageData.mType",
                "AnimatedImageData.mImage",
            ],
        ),
        (
            "network.skin_geometry",
            &[
                "SerializedSkinImpl.mGeometryData",
                "SerializedSkinImpl.mDefaultGeometryName",
                "JsonValue.toStyledString",
                "core.operator_delete",
            ],
        ),
        (
            "network.skin_details",
            &["SerializedSkinImpl.mIsPersona", "SerializedSkinImpl.mIsPremium", "SerializedSkinImpl.mResourcePatch"],
        ),
    ];
    for (id, entries) in rows {
        assert_eq!(entries_of(id), entries, "{id}");
        assert!(availability_of(entries, id).shape.is_some(), "{id}");
        for entry in entries {
            let without: Vec<&str> = entries.iter().copied().filter(|name| name != entry).collect();
            assert_eq!(availability_of(&without, id).missing, vec![*entry], "{id}");
        }
    }
    assert_eq!(AdapterCatalog::served_version(NeedKind::Event, "network.player_list"), Some(1));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "network.skin_details"), Some(1));
}

const CHAT_MEMBERS: [&str; 7] = [
    "GuiMessage.ctorShape",
    "GuiMessage.size",
    "GuiMessage.mType",
    "GuiMessage.mUsername",
    "GuiMessage.mFullString",
    "GuiMessage.mFilteredFullString",
    "GuiMessage.mFilteredFullStringEngaged",
];

fn chat_entries(constructor: &'static str) -> Vec<&'static str> {
    let mut entries = vec![constructor, "GuiMessage.mAuthorXuid"];
    entries.extend(CHAT_MEMBERS);
    entries
}

#[test]
fn the_chat_row_takes_the_flag_word_shape_when_both_constructors_resolve() {
    // `GuiMessage.ctor` still resolves on 70.x, so the flag word shape must come first.
    let mut both = chat_entries("GuiMessage.ctorWithFlagWord");
    both.push("GuiMessage.ctor");

    assert_eq!(availability_of(&both, "ui.chat_message").shape, Some(AdapterCatalog::CHAT_FLAG_WORD_SHAPE));
}

#[test]
fn the_chat_row_takes_the_plain_shape_without_the_flag_word_constructor() {
    let plain = chat_entries("GuiMessage.ctor");

    assert_eq!(availability_of(&plain, "ui.chat_message").shape, Some(AdapterCatalog::CHAT_CTOR_SHAPE));
}

#[test]
fn the_chat_row_needs_the_size_and_every_member_on_both_shapes() {
    for constructor in ["GuiMessage.ctor", "GuiMessage.ctorWithFlagWord"] {
        let entries = chat_entries(constructor);
        assert!(availability_of(&entries, "ui.chat_message").shape.is_some(), "{constructor}");
        for entry in &entries {
            let without: Vec<&str> = entries.iter().copied().filter(|name| name != entry).collect();
            assert_eq!(availability_of(&without, "ui.chat_message").shape, None, "{constructor} without {entry}");
        }
    }
}

#[test]
fn the_inline_icon_row_needs_the_width_entry_the_font_layout_and_the_texture_entries() {
    let all = entries_of("ui.inline_icons");
    for entry in [
        "BitmapFont._getCharWidth",
        "BitmapFont.vtable",
        "ClientInstance.getFontHandle",
        "FontHandle.dtor",
        "FontHandle.size",
        "FontHandle.mDefaultFont",
        "FontRepository.mLoadedFonts",
        "TextureGroup.uploadTexture",
        "TextureGroup.unloadTexture",
    ] {
        assert!(all.contains(&entry), "{entry}");
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "ui.inline_icons").missing, vec![entry]);
    }
    assert!(availability_of(&all, "ui.inline_icons").shape.is_some());
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "ui.inline_icons").unwrap();
    assert_eq!(need.kind, NeedKind::Area);
}

#[test]
fn the_textures_reloaded_event_needs_the_texture_entries_and_the_pack_hook() {
    let all: Vec<&str> =
        AdapterCatalog::needs().iter().flat_map(|need| need.shapes).flat_map(|shape| shape.entries).copied().collect();
    let without: Vec<&str> =
        all.iter().copied().filter(|name| *name != "MinecraftGame.applyActiveResourcePacks").collect();

    assert_eq!(availability_of(&all, "ui.textures_reloaded").shape, Some("default"));
    assert_eq!(
        availability_of(&without, "ui.textures_reloaded").missing,
        vec!["MinecraftGame.applyActiveResourcePacks"]
    );
    let without_group: Vec<&str> = all.iter().copied().filter(|name| *name != "TextureGroup.uploadTexture").collect();
    assert_eq!(availability_of(&without_group, "ui.textures_reloaded").missing, vec!["TextureGroup.uploadTexture"]);
}

#[test]
fn the_world_rendered_event_needs_the_level_pass_and_the_graphics_mode_read() {
    let all = ["LevelRenderer.renderLevel", "ClientInstance.getOptions", "IOptionRegistry.isGraphicsModeDeferred"];
    let row = availability_of(&all, "render.world_rendered");
    assert_eq!((row.kind, row.shape), (NeedKind::Event, Some("default")));
    assert_eq!(availability_of(&all[..2], "render.world_rendered").missing, ["IOptionRegistry.isGraphicsModeDeferred"]);
}

#[test]
fn the_world_canvas_area_needs_the_view_and_mesh_entries_but_not_render_mesh() {
    let row_entries: Vec<&str> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.id == "render.world_canvas")
        .flat_map(|need| need.shapes)
        .flat_map(|shape| shape.entries)
        .copied()
        .collect();
    let row = availability_of(&row_entries, "render.world_canvas");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 3));
    // `Mesh.renderMesh` fails on 70.26: the draw goes through the list overload only.
    assert!(!row_entries.contains(&"Mesh.renderMesh"));
    // The white texture of `draw` is a per-call need: the `ui.textures` entries are optional entries of the row.
    let optional: Vec<&str> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.id == "render.world_canvas")
        .flat_map(|need| need.shapes)
        .flat_map(|shape| shape.optional)
        .copied()
        .collect();
    let textures: Vec<&str> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.id == "ui.textures")
        .flat_map(|need| need.shapes)
        .flat_map(|shape| shape.entries)
        .copied()
        .collect();
    assert_eq!(optional, textures);
    for needed in [
        "FrameRenderObject.mViews",
        "ViewRenderObject.size",
        "LevelRendererCamera.mCameraPos",
        "Mesh.renderMeshTextureList",
        "Tessellator.mTessellating",
        "LevelRendererCamera.mWireframeMaterial",
        "BaseActorRenderer.size",
        "LevelRenderer.mAtlasTexture",
        "ClientInstance.getItemInHandRenderer",
        "ItemInHandRenderer.mMatBlendBlock",
    ] {
        let without: Vec<&str> = row_entries.iter().copied().filter(|name| *name != needed).collect();
        assert_eq!(availability_of(&without, "render.world_canvas").missing, [needed]);
    }
}

#[test]
fn the_camera_rows_need_the_setup_hook_the_writes_and_the_anchor() {
    let setup = availability_of(&["LevelRendererPlayer.setupCamera"], "camera.setup");
    assert_eq!((setup.kind, setup.shape), (NeedKind::Event, Some("default")));
    let pose = [
        "LevelRendererPlayer.createViewRenderObject",
        "LevelRendererCamera.preRenderUpdate",
        "LevelRendererCamera.mCameraPos",
        "LevelRendererCamera.mCameraTargetPos",
        "Camera.mForward",
        "Camera.updateViewMatrixDependencies",
        "ViewRenderObject.mViewData",
        "ViewRenderData.mCameraPos",
    ];
    let row = availability_of(&pose, "camera.pose");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 1));
    assert_eq!(availability_of(&pose[1..], "camera.pose").missing, ["LevelRendererPlayer.createViewRenderObject"]);
    let projection = availability_of(&["Camera.mForward"], "camera.projection");
    assert_eq!((projection.kind, projection.shape), (NeedKind::Area, Some("default")));
}

#[test]
fn the_active_camera_row_has_one_shape_for_each_component_size_entry() {
    let common = [
        "ClientInstance.getCameraRegistry",
        "CameraRegistry.mCameraEntities",
        "CameraRegistry.cameraEntityStride",
        "CameraComponent.typeId",
        "ActiveCameraComponent.typeId",
        "HashedString.layout",
    ];
    let with = |extra: &[&'static str]| -> Vec<&'static str> { common.iter().chain(extra).copied().collect() };

    let factor = availability_of(&with(&["CameraComponent.sizeFactor", "CameraComponent.sizeShift"]), "camera.active");
    assert_eq!((factor.kind, factor.shape, factor.version), (NeedKind::Area, Some("factor_shift"), 1));
    let size = availability_of(&with(&["CameraComponent.size"]), "camera.active");
    assert_eq!((size.kind, size.shape, size.version), (NeedKind::Area, Some("size"), 1));

    let neither = availability_of(&common, "camera.active");
    assert_eq!(neither.shape, None);
    assert_eq!(neither.missing, ["CameraComponent.sizeFactor", "CameraComponent.sizeShift"]);
    let half = availability_of(&with(&["CameraComponent.sizeFactor"]), "camera.active");
    assert_eq!(half.missing, ["CameraComponent.sizeShift"]);
    for needed in common {
        let without: Vec<&str> = with(&["CameraComponent.size"]).into_iter().filter(|name| *name != needed).collect();
        assert_eq!(availability_of(&without, "camera.active").shape, None, "{needed}");
    }
}

#[test]
fn the_free_camera_rows_need_the_handler_slots_the_packet_and_the_presets() {
    let free = entries_of("camera.free");
    assert_eq!(free.len(), 13);
    for needed in [
        "ClientNetworkHandler.handle.CameraInstructionPacket",
        "ClientNetworkHandler.handle.NetworkChunkPublisherUpdatePacket",
        "MinecraftPackets.createPacket",
        "Level.getCameraPresets",
        "CameraPreset.size",
        "Actor.mLevel",
        "ClientExitLevel.hook",
        "ClientInstance.onStartJoinGame",
        AdapterCatalog::CLIENT_INSTANCE_VTABLE,
    ] {
        assert!(free.contains(&needed), "{needed}");
        let without: Vec<&str> = free.iter().copied().filter(|name| *name != needed).collect();
        assert_eq!(availability_of(&without, "camera.free").missing, [needed]);
    }
    let row = availability_of(&free, "camera.free");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 1));

    let received = entries_of("camera.instruction_received");
    let event = availability_of(&received, "camera.instruction_received");
    assert_eq!((event.kind, event.shape), (NeedKind::Event, Some("default")));
    assert!(!received.contains(&"MinecraftPackets.createPacket"));
    assert!(received.contains(&"CameraInstruction.mSpline"));
}

#[test]
fn the_shapes_row_needs_the_capture_the_session_clearing_the_shapes_slot_and_the_payload() {
    let shapes = entries_of("world.shapes");
    for needed in [
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
    ] {
        assert!(shapes.contains(&needed), "{needed}");
        let without: Vec<&str> = shapes.iter().copied().filter(|name| *name != needed).collect();
        assert_eq!(availability_of(&without, "world.shapes").missing, [needed]);
    }
    let row = availability_of(&shapes, "world.shapes");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 1));
}

#[test]
fn focus_rows_need_their_entries() {
    let lost = availability_of(&[], "client.app_focus_lost");
    assert_eq!(lost.missing, vec!["MinecraftGame.onAppFocusLost"]);
    assert_eq!(availability_of(&["MinecraftGame.onAppFocusLost"], "client.app_focus_lost").shape, Some("default"));

    let gained = availability_of(&["MinecraftGame.onAppFocusGained"], "client.app_focus_gained");
    assert_eq!(gained.shape, None);
    assert_eq!(
        gained.missing,
        vec!["MinecraftGame.AppPlatformListener.vtable", "MinecraftGame.appPlatformListenerOffset"]
    );

    let replay = availability_of(&["MinecraftGame.onAppFocusLost"], "client.focus_replay");
    assert_eq!(replay.shape, Some("default"));
    assert_eq!(replay.kind, NeedKind::Area);
}

#[test]
fn options_rows_need_their_entries() {
    let distance = availability_of(&[], "options.view_distance");
    assert_eq!(distance.kind, NeedKind::Area);
    assert_eq!(distance.missing, vec!["ClientInstance.getOptions", "IOptionRegistry.getViewDistanceChunks"]);
    let both = ["ClientInstance.getOptions", "IOptionRegistry.getViewDistanceChunks"];
    assert_eq!(availability_of(&both, "options.view_distance").shape, Some("default"));

    let ints = availability_of(&["ClientInstance.getOptions"], "options.int");
    assert_eq!(ints.shape, None);
    assert_eq!(
        ints.missing,
        vec!["OptionRegistry.getIfValid", "OptionRegistry.getEffectiveMaxFramerateOption", "IntOption.set"]
    );
    let all = [
        "ClientInstance.getOptions",
        "OptionRegistry.getIfValid",
        "OptionRegistry.getEffectiveMaxFramerateOption",
        "IntOption.set",
    ];
    let row = availability_of(&all, "options.int");
    assert_eq!((row.kind, row.shape, row.version), (NeedKind::Area, Some("default"), 1));
}

#[test]
fn block_states_row_needs_the_serialization_member() {
    let all = entries_of("world.block_states");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "world.block_states"), Some(1));
    assert!(availability_of(&all, "world.block_states").shape.is_some());
    assert!(!all.contains(&"BlockTypeRegistry.mRegistry"));
    for entry in [
        "BlockTypeRegistry.instance",
        "BlockTypeRegistry.lookupByName",
        "Block.mSerializationId",
        "StringByteInput.vtable",
        "NbtIo.read",
        "BlockSerializationUtils.tryGetBlockFromNBT",
        "CompoundTag.dtor",
        "VanillaBlockStateTransformUtils.transformBlock",
        "BlockType.asItemInstance",
        "ItemStackBase.dtor",
    ] {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "world.block_states").missing, vec![entry]);
    }
}

#[test]
fn blocks_row_needs_the_block_read_entries_and_the_block_source_chain() {
    let all = entries_of("world.blocks");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "world.blocks"), Some(1));
    assert!(availability_of(&all, "world.blocks").shape.is_some());
    for entry in [
        "BlockSource.getBlock",
        "Block.mBlockType",
        "BlockType.canBeBuiltOver",
        "BlockType.isInteractiveBlock",
        "BlockType.asItemInstance",
        "BlockType.mNameInfo.mFullName",
        "ItemStackBase.dtor",
        "HashedString.layout",
        "Dimension.getBlockSourceFromMainChunkSource",
    ] {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "world.blocks").missing, vec![entry]);
    }
}

#[test]
fn kinds_in_box_row_needs_the_chunk_and_block_reads_and_the_block_source_chain() {
    let all = entries_of("world.kinds_in_box");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "world.kinds_in_box"), Some(1));
    assert!(availability_of(&all, "world.kinds_in_box").shape.is_some());
    for entry in [
        "BlockSource.getChunk",
        "BlockSource.getBlock",
        "LevelChunk.mSubChunks",
        "SubChunk.isPlaceHolderSubChunk",
        "Block.mBlockType",
        "BlockType.mNameInfo.mFullName",
        "HashedString.layout",
        "Dimension.getBlockSourceFromMainChunkSource",
    ] {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "world.kinds_in_box").missing, vec![entry]);
    }
}

#[test]
fn block_mesher_row_needs_the_block_tessellator_and_the_block_reads() {
    let all = entries_of("render.block_mesher");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "render.block_mesher"), Some(1));
    assert!(availability_of(&all, "render.block_mesher").shape.is_some());
    for entry in [
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
        "Dimension.getBlockSourceFromMainChunkSource",
        "BlockTypeRegistry.lookupByName",
        "HashedString.layout",
    ] {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.block_mesher").missing, vec![entry]);
    }
}

#[test]
fn block_mesher_row_needs_the_mesh_data_and_the_tint_entries() {
    let all = entries_of("render.block_mesher");
    for entry in [
        "MeshData.clear",
        "MeshData.dtor",
        "MeshData.moveAssign",
        "BiomeColorSampling.getTessellationPolicy",
        "TessellationPolicy.get",
        "BlockType.mTintMethod",
    ] {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.block_mesher").missing, vec![entry]);
    }
}

#[test]
fn water_tint_row_needs_the_chunk_the_biome_and_the_water_colour() {
    let all = entries_of("render.water_tint");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "render.water_tint"), Some(1));
    assert_eq!(all, ["BlockSource.getChunk", "LevelChunk.getBiome", "BiomeColorSampling.getWaterColor"]);
    assert!(availability_of(&all, "render.water_tint").shape.is_some());
    for entry in &all {
        let without: Vec<&str> = all.iter().copied().filter(|name| name != entry).collect();
        assert_eq!(availability_of(&without, "render.water_tint").missing, vec![*entry]);
    }
}

#[test]
fn block_changed_is_served_at_version_2_with_the_entries_of_version_1() {
    let all = entries_of("world.block_changed");
    assert_eq!(AdapterCatalog::served_version(NeedKind::Event, "world.block_changed"), Some(2));
    assert_eq!(availability_of(&all, "world.block_changed").version, 2);
}

#[test]
fn use_pressed_needs_the_build_action_handler() {
    let row = availability_of(&["ClientInputCallbacks.handleBuildAction"], "player.use_pressed");
    assert_eq!((row.kind, row.shape), (NeedKind::Event, Some("default")));
    assert_eq!(availability_of(&[], "player.use_pressed").missing, ["ClientInputCallbacks.handleBuildAction"]);
}

#[test]
fn build_block_needs_the_game_mode_slot_and_its_player() {
    let all = ["GameMode.vtable", "GameMode.buildBlock", "GameMode.mPlayer", "ClientInstance.getLocalPlayer"];
    assert_eq!(entries_of("player.build_block"), all);
    let row = availability_of(&all, "player.build_block");
    assert_eq!((row.kind, row.shape), (NeedKind::Event, Some("default")));
    for entry in all {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "player.build_block").missing, vec![entry]);
    }
}

#[test]
fn player_build_needs_the_game_mode_the_hit_result_and_the_delay() {
    let all =
        ["Player.mGameMode", "Actor.mLevel", "Level.getHitResult", "GameMode.buildBlock", "GameMode.buildingTickDelay"];
    assert_eq!(entries_of("player.build"), all);
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "player.build"), Some(1));
    let row = availability_of(&all, "player.build");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    for entry in all {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "player.build").missing, vec![entry]);
    }
}

#[test]
fn the_pick_range_row_has_one_shape_per_build() {
    let getter = availability_of(&["Player.mGameMode", "GameMode.getMaxPickRange"], "player.pick_range");
    assert_eq!((getter.kind, getter.shape), (NeedKind::Area, Some(AdapterCatalog::PICK_RANGE_GETTER_SHAPE)));
    let both =
        availability_of(&["Player.mGameMode", "GameMode.getMaxPickRange", "Player.isCreative"], "player.pick_range");
    assert_eq!(both.shape, Some(AdapterCatalog::PICK_RANGE_GETTER_SHAPE));
    let creative = availability_of(&["Player.mGameMode", "Player.isCreative"], "player.pick_range");
    assert_eq!(creative.shape, Some(AdapterCatalog::PICK_RANGE_CREATIVE_SHAPE));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "player.pick_range"), Some(1));
    let neither = availability_of(&[], "player.pick_range");
    assert_eq!(neither.shape, None);
    assert_eq!(neither.missing, ["Player.mGameMode", "GameMode.getMaxPickRange"]);
}

#[test]
fn the_block_entity_pass_needs_the_body_hook_and_the_player_vtable() {
    let all = ["LevelRendererCamera.renderBlockEntitiesBody", "LevelRendererPlayer.vtable"];
    assert_eq!(entries_of("render.block_entity_pass"), all);
    let row = availability_of(&all, "render.block_entity_pass");
    assert_eq!((row.kind, row.shape), (NeedKind::Event, Some("default")));
    for entry in all {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.block_entity_pass").missing, vec![entry]);
    }
}

// The entries both shapes of `render.block_entities` need besides the factory.
const BLOCK_ENTITY_COMMON: [&str; 20] = [
    "LevelRendererCamera.renderBlockEntitiesBody",
    "LevelRendererPlayer.vtable",
    "BlockActor.loadStatic",
    "BlockActor._getRenderComponent",
    "BlockActor.mPosition",
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
];

#[test]
fn the_block_entities_row_has_one_factory_shape_per_build() {
    let mut factory = BLOCK_ENTITY_COMMON.to_vec();
    factory.push("VanillaBlockActorFactory.createBlockActor");
    let row = availability_of(&factory, "render.block_entities");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some(AdapterCatalog::BLOCK_ENTITIES_FACTORY_SHAPE)));
    let mut flag = BLOCK_ENTITY_COMMON.to_vec();
    flag.extend(["VanillaBlockActorFactory.createBlockActorWithFlag", "Level.isClientSide"]);
    let row = availability_of(&flag, "render.block_entities");
    assert_eq!(row.shape, Some(AdapterCatalog::BLOCK_ENTITIES_FLAG_SHAPE));
    // The flag form needs the level's flag slot as well.
    flag.retain(|name| *name != "Level.isClientSide");
    assert_eq!(availability_of(&flag, "render.block_entities").shape, None);
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "render.block_entities"), Some(1));
    for entry in BLOCK_ENTITY_COMMON {
        let without: Vec<&str> = factory.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.block_entities").missing, vec![entry]);
    }
}

#[test]
fn the_chest_pair_row_needs_the_type_test_the_position_and_the_chest_members() {
    let all = [
        "BlockActor.isType",
        "BlockActor.mPosition",
        "ChestBlockActor.mPairFlags",
        "ChestBlockActor.mLargeChestPaired",
        "ChestBlockActor.mLargeChestPairedPosition",
    ];
    assert_eq!(entries_of("render.chest_pair"), all);
    let row = availability_of(&all, "render.chest_pair");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some("default")));
    for entry in all {
        let without: Vec<&str> = all.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.chest_pair").missing, vec![entry]);
    }
}

const SKULL_COMMON: [&str; 4] = [
    "BlockActorRenderDispatcher.mSkullRenderer",
    "BlockType.mNameInfo.mFullName",
    "HashedString.layout",
    "BlockActor.isType",
];

#[test]
fn the_skull_pose_row_has_one_shape_per_build() {
    let mut render_skull = SKULL_COMMON.to_vec();
    render_skull.push("SkullBlockRenderer.renderSkull");
    let row = availability_of(&render_skull, "render.skull_pose");
    assert_eq!((row.kind, row.shape), (NeedKind::Area, Some(AdapterCatalog::SKULL_RENDER_SHAPE)));
    let mut upright = SKULL_COMMON.to_vec();
    upright.extend([
        "SkullBlockRenderer.renderSkullUpright",
        "MatrixStack.push",
        "MatrixStackRef.dtor",
        "ScreenContext.camera",
        "Camera.mForward",
    ]);
    let row = availability_of(&upright, "render.skull_pose");
    assert_eq!(row.shape, Some(AdapterCatalog::SKULL_UPRIGHT_SHAPE));
    // The upright form draws inside a pushed matrix.
    upright.retain(|name| *name != "MatrixStack.push");
    assert_eq!(availability_of(&upright, "render.skull_pose").shape, None);
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "render.skull_pose"), Some(1));
    for entry in SKULL_COMMON {
        let without: Vec<&str> = render_skull.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "render.skull_pose").missing, vec![entry]);
    }
}

#[test]
fn the_wire_rows_need_the_hooks_the_connection_walk_and_the_protocol() {
    const WIRE: [&str; 11] = [
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
    ];
    for event in ["network.packet_wire", "network.connection_started"] {
        assert_eq!(entries_of(event), WIRE);
        assert_eq!(availability_of(&WIRE, event).kind, NeedKind::Event);
        assert!(availability_of(&WIRE, event).shape.is_some());
        let without: Vec<&str> = WIRE.iter().copied().filter(|name| *name != "NetworkConnection.mType").collect();
        assert_eq!(availability_of(&without, event).missing, vec!["NetworkConnection.mType"]);
    }
}

#[test]
fn the_chunk_cache_and_protocol_rows_need_their_entries() {
    const CACHE: [&str; 3] =
        ["NetworkSystem.send", "ClientCacheStatusPacket.vtable", "ClientCacheStatusPacket.mEnabled"];
    assert_eq!(entries_of("network.chunk_cache"), CACHE);
    assert_eq!(availability_of(&CACHE, "network.chunk_cache").kind, NeedKind::Area);
    assert!(availability_of(&CACHE, "network.chunk_cache").shape.is_some());
    for entry in CACHE {
        let without: Vec<&str> = CACHE.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "network.chunk_cache").missing, vec![entry]);
    }
    assert_eq!(entries_of("network.protocol"), ["SharedConstants.NetworkProtocolVersion"]);
    assert!(availability_of(&["SharedConstants.NetworkProtocolVersion"], "network.protocol").shape.is_some());
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "network.chunk_cache"), Some(1));
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "network.protocol"), Some(1));
}

#[test]
fn the_time_rows_need_their_entries() {
    const TIME: [&str; 11] = [
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
    ];
    assert_eq!(entries_of("time.control"), TIME);
    assert_eq!(entries_of("time.step_applied"), TIME);
    assert_eq!(availability_of(&TIME, "time.control").kind, NeedKind::Area);
    assert_eq!(availability_of(&TIME, "time.step_applied").kind, NeedKind::Event);
    for entry in TIME {
        let without: Vec<&str> = TIME.iter().copied().filter(|name| *name != entry).collect();
        assert_eq!(availability_of(&without, "time.control").missing, vec![entry]);
    }
    assert_eq!(AdapterCatalog::served_version(NeedKind::Area, "time.control"), Some(1));
}

#[test]
fn the_frame_end_and_latency_barrier_rows_need_their_entries() {
    const FRAME_END: [&str; 1] = ["MinecraftGame.endFrame"];
    const BARRIER: [&str; 2] =
        ["LegacyClientNetworkHandler.vtable", "ClientNetworkHandler.handle.NetworkStackLatencyPacket"];
    assert_eq!(entries_of("time.frame_end"), FRAME_END);
    assert_eq!(entries_of("time.latency_barrier"), BARRIER);
    assert!(availability_of(&FRAME_END, "time.frame_end").shape.is_some());
    assert!(availability_of(&BARRIER, "time.latency_barrier").shape.is_some());
    assert_eq!(availability_of(&[], "time.frame_end").missing, FRAME_END);
    assert_eq!(availability_of(&BARRIER[..1], "time.latency_barrier").missing, vec![BARRIER[1]]);
}

#[test]
fn the_hide_actor_and_player_moved_rows_need_their_entries() {
    const HIDE: [&str; 5] = [
        "LevelRendererPlayer.vtable",
        "LevelRendererPlayer.queueRenderEntities",
        "LevelRendererCamera.mActorRenderQueue",
        "ActorRenderDispatcher.render",
        "ClientInstance.getLocalPlayer",
    ];
    const MOVED: [&str; 4] = [
        "LegacyClientNetworkHandler.vtable",
        "ClientNetworkHandler.handle.MovePlayerPacket",
        "Actor.getRuntimeID",
        "ClientInstance.getLocalPlayer",
    ];
    assert_eq!(entries_of("camera.hide_actor"), HIDE);
    assert_eq!(entries_of("player.moved"), MOVED);
    assert!(availability_of(&HIDE, "camera.hide_actor").shape.is_some());
    assert!(availability_of(&MOVED, "player.moved").shape.is_some());
    assert_eq!(availability_of(&HIDE[..3], "camera.hide_actor").missing, vec![HIDE[3], HIDE[4]]);
    assert_eq!(availability_of(&MOVED[..2], "player.moved").missing, vec![MOVED[2], MOVED[3]]);
}

#[test]
fn server_join_starting_v2_needs_the_external_world_slot_and_join_multiplayer_with_address() {
    let all = entries_of("world.server_join_starting");
    for entry in [
        "ClientInstance.startExternalNetworkWorld",
        "ClientInstance.vtable",
        "MinecraftGame.joinMultiplayerWithAddress",
        "MinecraftGame.vtable",
        "GameConnectionInfo.size",
    ] {
        assert!(all.contains(&entry), "{entry}: {all:?}");
    }
    assert_eq!(AdapterCatalog::served_version(NeedKind::Event, "world.server_join_starting"), Some(2));
    let without: Vec<&str> =
        all.iter().copied().filter(|entry| *entry != "MinecraftGame.joinMultiplayerWithAddress").collect();
    let row = availability_of(&without, "world.server_join_starting");
    assert_eq!((row.shape, row.missing), (None, vec!["MinecraftGame.joinMultiplayerWithAddress"]));
}

#[test]
fn the_direct_join_area_needs_the_games_connect_by_address_and_the_client_vtable() {
    let all = entries_of("client.join_server_direct");
    assert_eq!(all, ["ClientInstance.connectByAddress", "ClientInstance.connectByAddressThis"]);
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "client.join_server_direct").unwrap();
    assert_eq!((need.kind, need.version), (NeedKind::Area, 1));
}

#[test]
fn the_title_rows_need_the_router_hooks_and_the_client_router() {
    assert_eq!(entries_of("client.title_screen"), ["OreUIRouter.pushRoute", "OreUIRouter.replaceRoute"]);
    assert_eq!(entries_of("client.go_to_title"), ["ClientInstance.getCurrentUIRouter", "OreUIRouter.pushRoute"]);
    for id in ["client.title_screen", "client.go_to_title"] {
        let need = AdapterCatalog::needs().iter().find(|need| need.id == id).unwrap();
        assert_eq!((need.kind, need.version), (NeedKind::Area, 1), "{id}");
    }
}

#[test]
fn the_hit_result_row_needs_the_level_of_the_player_and_its_hit_result_slot() {
    assert_eq!(entries_of("player.hit_result"), ["Actor.mLevel", "Level.getHitResult"]);
    let need = AdapterCatalog::needs().iter().find(|need| need.id == "player.hit_result").unwrap();
    assert_eq!((need.kind, need.version), (NeedKind::Area, 1));
}

use amml_abi::{
    AmmlUseRow, ChatMessageAdded, ClientTick, ContainerSlotHover, ContainerSlotUnhover, InventoryRequestDone,
    ItemHovertext, KeyInput, LocatorBarIconDraw, ModsLoaded, PlayerListUpdate, PlayerSkinChanged, ProviderGone,
    ProviderReady, ScreenChanged, ScreenClosing, ScreenRender, UiPointer, UiScroll, WorldLeft,
};
use amml_api::{
    ActorUniqueId, Amount, AnimatedKind, Area, Cancel, ChatKind, ChatMessageAddedExt, Client, Collection, Color,
    ContainerContents, ContainerEntry, ContainerScreen, ContainerSlotHoverExt, Ctx, HookOrder, ImageView, InlineIcon,
    Inventory, InventoryMove, InventoryRequestDoneExt, ItemDrawFlags, ItemDrawHandle, ItemHovertextExt, ItemSnapshot,
    ItemView, LocalPlayer, LocatorBarIconDrawExt, PixelFormat, PlayerListEntry, PlayerListUpdateExt,
    PlayerSkinChangedExt, Point, Rect, RequestId, RequestOptions, RequestOutcome, Rgba, ScreenChangedExt,
    ScreenClosingExt, ScreenId, ScreenRenderExt, ScreenView, SkinRef, SlotAt, TextAlign, TextureId, TextureSource,
    Tint, UiDraw, UiPointerExt, UiScrollExt, Uses, Uuid, Uv, ViewId,
};

const PEEK: Uses = amml_api::amml_uses! {
    feature "preview" {
        required ScreenRender; required ContainerSlotHover; required ContainerSlotUnhover; required ScreenClosing;
        required ProviderReady; required ProviderGone; required ModsLoaded;
        required area Area::UI_DRAW.at(2); required area Area::UI_SCREEN.at(2); required area Area::INVENTORY_READ.at(4);
    }
    feature "counts" { required area Area::UI_TEXT; }
    feature "bundle_pick" { required UiScroll; required KeyInput; }
    feature "hovertext" { required ItemHovertext; }
};

const HOTBAR_MANAGER: Uses = amml_api::amml_uses! {
    feature "core" {
        required ClientTick; required ProviderReady; required ProviderGone; required ModsLoaded;
        required area Area::CLIENT_READ;
    }
    feature "buttons" {
        required ScreenRender; required UiPointer;
        required area Area::UI_SCREEN; required area Area::UI_DRAW; required area Area::UI_TEXTURES;
    }
    feature "screen_actions" {
        required ScreenRender; required ScreenChanged; required area Area::UI_SCREEN.at(2);
        required area Area::CLIENT_TOP_SCREEN;
    }
    feature "rotation" {
        required InventoryRequestDone; required area Area::INVENTORY_READ.at(3); required area Area::INVENTORY_REQUEST;
        required area Area::CLIENT_TOP_SCREEN;
    }
};

const HEADS: Uses = amml_api::amml_uses! {
    feature "faces" {
        required PlayerListUpdate; required PlayerSkinChanged; required LocatorBarIconDraw; required WorldLeft;
        required area Area::UI_TEXTURES; required area Area::NETWORK_SKIN;
    }
    feature "settings" { required ClientTick; required ProviderReady; required ProviderGone; required ModsLoaded; }
    feature "chat" { required ChatMessageAdded; required area Area::UI_INLINE_ICONS; }
    feature "persona" { required area Area::NETWORK_SKIN_ANIMATED; }
    feature "geometry" { required area Area::NETWORK_SKIN_GEOMETRY; }
    feature "diagnostics" { required area Area::NETWORK_SKIN_DETAILS; }
};

type Row = (String, u8, String, u32, bool);

fn text(view: amml_abi::AmmlStr) -> String {
    // SAFETY: the rows of `amml_uses!` point to static strings.
    unsafe { view.as_str() }.unwrap().to_string()
}

// (feature, kind, id, version, required) per row.
fn rows(uses: &Uses) -> Vec<Row> {
    uses.rows()
        .iter()
        .map(|row: &AmmlUseRow| (text(row.feature), row.kind, text(row.name), row.version, row.required != 0))
        .collect()
}

fn ids(uses: &Uses, kind: u8) -> Vec<String> {
    rows(uses).into_iter().filter(|row| row.1 == kind).map(|row| row.2).collect()
}

#[test]
fn contract_uses_blocks_compile_and_list_every_row() {
    assert_eq!(
        ids(&PEEK, AmmlUseRow::KIND_EVENT),
        [
            "ui.screen_render",
            "ui.container_slot_hover",
            "ui.container_slot_unhover",
            "ui.screen_closing",
            "registry.provider_ready",
            "registry.provider_gone",
            "mods.loaded",
            "input.ui_scroll",
            "input.key",
            "ui.item_hovertext"
        ]
    );
    assert_eq!(ids(&PEEK, AmmlUseRow::KIND_AREA), ["ui.draw", "ui.screen", "inventory.read", "ui.text"]);
    assert_eq!(
        ids(&HOTBAR_MANAGER, AmmlUseRow::KIND_EVENT),
        [
            "client.tick",
            "registry.provider_ready",
            "registry.provider_gone",
            "mods.loaded",
            "ui.screen_render",
            "input.ui_pointer",
            "ui.screen_render",
            "ui.screen_changed",
            "inventory.request_done"
        ]
    );
    assert_eq!(
        ids(&HOTBAR_MANAGER, AmmlUseRow::KIND_AREA),
        [
            "client.read",
            "ui.screen",
            "ui.draw",
            "ui.textures",
            "ui.screen",
            "client.top_screen",
            "inventory.read",
            "inventory.request",
            "client.top_screen"
        ]
    );
    assert_eq!(
        ids(&HEADS, AmmlUseRow::KIND_EVENT),
        [
            "network.player_list",
            "network.player_skin",
            "ui.locator_bar_icon",
            "world.left",
            "client.tick",
            "registry.provider_ready",
            "registry.provider_gone",
            "mods.loaded",
            "ui.chat_message"
        ]
    );
    assert_eq!(
        ids(&HEADS, AmmlUseRow::KIND_AREA),
        [
            "ui.textures",
            "network.skin",
            "ui.inline_icons",
            "network.skin_animated",
            "network.skin_geometry",
            "network.skin_details"
        ]
    );
    for uses in [&PEEK, &HOTBAR_MANAGER, &HEADS] {
        assert!(rows(uses).iter().all(|row| row.4), "every row is required");
    }
}

// Names the calls and types of the contract: a rename or a removed method fails the build of this file.
#[test]
fn contract_calls_exist() {
    let _ = HookOrder::READ_AFTER_WRITERS;
    let _ = (
        ScreenView::pointer,
        ScreenView::size,
        ScreenView::container_screen,
        ScreenView::kind,
        ScreenView::content_rect,
        ScreenView::input_mode,
        ScreenView::ui_profile,
    );
    let _ = (ContainerScreen::id, ContainerScreen::kind, ContainerScreen::slots, ContainerScreen::cursor);
    let _ = (ContainerScreen::visual_item, ContainerScreen::auto_place, ContainerScreen::take_all);
    let _ = (ContainerScreen::place_all, ContainerScreen::drop_slot, ContainerScreen::drop_cursor);
    let _ = ContainerScreen::swap;
    let _ = (ItemView::type_name, ItemView::count, ItemView::is_empty, ItemView::identity);
    let _ = (ItemView::container_contents, ItemView::snapshot);
    let _ = (UiDraw::fill_rect, UiDraw::outline_rect, UiDraw::item, UiDraw::text, UiDraw::text_width);
    let _ = (UiDraw::image, UiDraw::gui_scale, UiDraw::can_draw);
    let _ = (Client::local_player, Client::top_screen, LocalPlayer::inventory);
    let _ = (Inventory::slots, Inventory::request_moves, Inventory::request_swap);
    let _ = (Ctx::input, Ctx::textures, Ctx::chat);
    let _ = (SkinRef::image, SkinRef::animated_images, SkinRef::is_persona, SkinRef::default_geometry_name);
    let _ = (SkinRef::geometry_json, SkinRef::is_premium, SkinRef::resource_patch);
    let _ = (amml_api::Textures::load, amml_api::Textures::ready, amml_api::Textures::unload);
    let _ = amml_api::Textures::upload_rgba;
    let _ = amml_api::Chat::register_inline_icon;
    let _ = (InlineIcon::glyph, InlineIcon::update, InlineIcon::release);
    let _ = ItemDrawFlags::FOIL_ONLY;
    let _ = TextureSource::ModFile("textures/sort.png");
    let _ = TextAlign::Left;
    let _ = RequestOutcome::NotSent;

    let snapshot = ItemSnapshot::default();
    let _ = (&snapshot.damage, &snapshot.max_damage, &snapshot.is_glint, &snapshot.type_name, snapshot.count);
    let _ = (&snapshot.aux, &snapshot.max_count, &snapshot.identity);

    let _ = |entry: ContainerEntry| -> (u8, ItemSnapshot, ItemDrawHandle, Option<ContainerContents>, Option<i32>) {
        (entry.slot, entry.item, entry.draw, entry.bundle, entry.bundle_weight)
    };
    let _ = |image: ImageView<'_>| (image.width, image.height, image.format, image.pixels.len(), PixelFormat::Rgba8);

    let _ = <ScreenRender as ScreenRenderExt>::screen;
    let _ = <ScreenRender as ScreenRenderExt>::view_id;
    let _ = <ScreenRender as ScreenRenderExt>::view;
    let _ = <ScreenRender as ScreenRenderExt>::draw;
    let _ = <ScreenRender as ScreenRenderExt>::container_screen;
    let _ = <ScreenRender as ScreenRenderExt>::tick;
    let _ = <ContainerSlotHover as ContainerSlotHoverExt>::screen;
    let _ = <ContainerSlotHover as ContainerSlotHoverExt>::slot;
    let _ = <ContainerSlotUnhover as ContainerSlotHoverExt>::slot;
    let _ = <ScreenClosing as ScreenClosingExt>::screen;
    let _ = <UiScroll as UiScrollExt>::screen;
    let _ = <UiScroll as UiScrollExt>::notches;
    let _ = <UiScroll as Cancel>::cancel;
    let _ = <ItemHovertext as ItemHovertextExt>::kind;
    let _ = <ItemHovertext as ItemHovertextExt>::hide_contents;
    let _ = <UiPointer as UiPointerExt>::pressed;
    let _ = <UiPointer as UiPointerExt>::position;
    let _ = <UiPointer as UiPointerExt>::input_mode;
    let _ = <UiPointer as Cancel>::cancel;
    let _ = <ScreenChanged as ScreenChangedExt>::old;
    let _ = <ScreenChanged as ScreenChangedExt>::new;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::outcome;
    let _ = <InventoryRequestDone as InventoryRequestDoneExt>::accepted;
    let _ = <PlayerListUpdate as PlayerListUpdateExt>::entries;
    let _ = <PlayerListUpdate as PlayerListUpdateExt>::action;
    let _ = <PlayerSkinChanged as PlayerSkinChangedExt>::uuid;
    let _ = <PlayerSkinChanged as PlayerSkinChangedExt>::skin;
    let _ = <LocatorBarIconDraw as LocatorBarIconDrawExt>::replace_texture;
    let _ = <LocatorBarIconDraw as LocatorBarIconDrawExt>::set_rect;
    let _ = <LocatorBarIconDraw as LocatorBarIconDrawExt>::set_color;
    let _ = <ChatMessageAdded as ChatMessageAddedExt>::set_text;
    let _ = <ChatMessageAdded as ChatMessageAddedExt>::set_filtered_text;
    let _ = <ChatMessageAdded as ChatMessageAddedExt>::inline_icons;
    let _ = PlayerListEntry::actor_id;
    let _ = PlayerListEntry::skin;
    let _ = PlayerListEntry::xuid;

    // Names only: the types of the contract exist.
    let _ = std::marker::PhantomData::<(
        ActorUniqueId,
        AnimatedKind,
        ChatKind,
        Collection,
        Color,
        InventoryMove,
        RequestId,
        RequestOptions,
        Rgba,
        Point,
        Rect,
        ScreenId,
        SlotAt,
        TextureId,
        Tint,
        Uuid,
        ViewId,
        Amount,
        Uv,
    )>;
}

use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use amml_game::{EntryId, GameEntries, HookOrder};
use amml_sdk::{AmmlMod, Api, Logger, SdkError};

use crate::{
    NameTagEntries, PEEK, PROBE, PeekEntries, Probe, ProbeReport, client_update, controller_dtor,
    create_view_render_object, extract_name_tags, screen_view_render, setup_camera, slot_hovered, slot_unhovered,
    ui_scene_direction,
};

/// Hooks `ClientInstance.update` and logs `amml-testmod: ticks=100` once, after the hooked function
/// has run 100 times. It also reads the player pose, the dimension and the camera through `amml-game`.
pub struct TestMod;

impl TestMod {
    // The five entries the camera feature needs; `None` when any is off.
    fn camera_targets(entries: &GameEntries) -> Option<(u64, u64)> {
        entries.get(EntryId::CAMERA_FORWARD)?;
        entries.get(EntryId::VIEW_RENDER_OBJECT_VIEW_DATA)?;
        entries.get(EntryId::VIEW_RENDER_DATA_CAMERA_POS)?;
        Some((
            entries.get(EntryId::LEVEL_RENDERER_PLAYER_SETUP_CAMERA)?,
            entries.get(EntryId::LEVEL_RENDERER_PLAYER_CREATE_VIEW_RENDER_OBJECT)?,
        ))
    }

    // The five peek hooks that the resolved entries allow: the two slot callbacks and the controller destructor
    // need the container feature, the render function needs the UI feature, the wheel function needs the scroll
    // feature. A hook whose feature is off is not installed.
    fn peek_hooks(entries: &GameEntries) -> Vec<(&'static str, u64, *const c_void, &'static AtomicPtr<c_void>)> {
        let mut hooks = Vec::new();
        if let (Some(hovered), Some(unhovered), Some(dtor)) = (
            entries.get(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_HOVERED),
            entries.get(EntryId::CONTAINER_SCREEN_CONTROLLER_ON_CONTAINER_SLOT_UNHOVERED),
            entries.get(EntryId::SCREEN_CONTROLLER_DTOR),
        ) && entries.get(EntryId::CONTAINER_SCREEN_CONTROLLER_GET_VISUAL_ITEM_STACK_IMPL).is_some()
        {
            hooks.push((
                "ContainerScreenController.onContainerSlotHovered",
                hovered,
                slot_hovered::detour as *const c_void,
                slot_hovered::GUARD.original(),
            ));
            hooks.push((
                "ContainerScreenController.onContainerSlotUnhovered",
                unhovered,
                slot_unhovered::detour as *const c_void,
                slot_unhovered::GUARD.original(),
            ));
            hooks.push((
                "ScreenController.dtor",
                dtor,
                controller_dtor::detour as *const c_void,
                controller_dtor::GUARD.original(),
            ));
        }
        if let Some(render) = entries.get(EntryId::SCREEN_VIEW_RENDER)
            && entries.get(EntryId::SCREEN_VIEW_POINTER_LOCATION_PREVIOUS).is_some()
        {
            hooks.push((
                "ScreenView.render",
                render,
                screen_view_render::detour as *const c_void,
                screen_view_render::GUARD.original(),
            ));
        }
        if let Some(direction) = entries.get(EntryId::UI_SCENE_HANDLE_DIRECTION)
            && entries.get(EntryId::UI_SCENE_SCREEN_VIEW).is_some()
        {
            hooks.push((
                "UIScene.handleDirection",
                direction,
                ui_scene_direction::detour as *const c_void,
                ui_scene_direction::GUARD.original(),
            ));
        }
        hooks
    }

    // Installs one hook on `target` and logs it.
    #[allow(unsafe_code)]
    fn hook(
        api: &Api,
        name: &str,
        target: u64,
        detour: *const c_void,
        original: &'static AtomicPtr<c_void>,
    ) -> Result<(), SdkError> {
        // SAFETY: `target` is the start of the game function `name`, which `detour` has the signature of (its
        // module documents it), and `original` belongs to this one hook.
        let hook = unsafe { api.hook(target, detour, HookOrder::READ_AFTER_WRITERS, original) }?;
        Logger::new(api, &Self::id()).info(&format!(
            "hooked {name} at {target:#x} (rva {:#x}), handle {}",
            target.wrapping_sub(api.module_base()),
            hook.0
        ));
        Ok(())
    }
}

impl AmmlMod for TestMod {
    const NAMESPACE: &'static str = "amml";
    const NAME: &'static str = "testmod";

    fn enable(api: &Api) -> Result<(), SdkError> {
        let entries = GameEntries::resolve(api, &Probe::wanted()).map_err(|error| SdkError::Mod(error.to_string()))?;
        Logger::new(api, &Self::id()).info(&ProbeReport::features_line(&entries.off_features()));
        let update = entries
            .get(EntryId::CLIENT_INSTANCE_UPDATE)
            .ok_or_else(|| SdkError::MissingEntry(EntryId::CLIENT_INSTANCE_UPDATE.as_str().to_string()))?;
        let peek =
            GameEntries::resolve(api, &PeekEntries::wanted()).map_err(|error| SdkError::Mod(error.to_string()))?;
        Logger::new(api, &Self::id()).info(&ProbeReport::peek_features_line(&peek.off_features()));
        let peek_hooks = Self::peek_hooks(&peek);
        // The peek probe holds its entries before the first peek hook can run.
        PEEK.start(peek);
        let camera = Self::camera_targets(&entries);
        let name_tags = NameTagEntries::target(&entries).filter(|_| entries.get(NameTagEntries::SIZE).is_some());
        // The probe holds the entries before the first hook can run.
        PROBE.start(entries);
        Self::hook(
            api,
            EntryId::CLIENT_INSTANCE_UPDATE.as_str(),
            update,
            client_update::detour as *const c_void,
            client_update::GUARD.original(),
        )?;
        if let Some((setup, create)) = camera {
            Self::hook(
                api,
                "LevelRendererPlayer.setupCamera",
                setup,
                setup_camera::detour as *const c_void,
                setup_camera::GUARD.original(),
            )?;
            Self::hook(
                api,
                "LevelRendererPlayer.createViewRenderObject",
                create,
                create_view_render_object::detour as *const c_void,
                create_view_render_object::GUARD.original(),
            )?;
        }
        if let Some(target) = name_tags {
            Self::hook(
                api,
                "LevelRendererPlayer.extractNameTags",
                target,
                extract_name_tags::detour as *const c_void,
                extract_name_tags::GUARD.original(),
            )?;
        }
        for (name, target, detour, original) in peek_hooks {
            Self::hook(api, name, target, detour, original)?;
        }
        Ok(())
    }

    fn disable(api: &Api) {
        PEEK.stop();
        PROBE.stop();
        Logger::new(api, &Self::id()).info("disabled");
    }
}

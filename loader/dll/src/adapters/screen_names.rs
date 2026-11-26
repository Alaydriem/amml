use amml_abi::AmmlScreen;

/// Maps the game's top screen names to `AmmlScreen`, so a mod never sees a game screen name.
pub struct ScreenNames;

impl ScreenNames {
    // The JSON UI screens that show a container next to the player's inventory.
    const CONTAINERS: [&'static str; 26] = [
        "inventory_screen",
        "chest_screen",
        "small_chest_screen",
        "large_chest_screen",
        "ender_chest_screen",
        "shulker_box_screen",
        "barrel_screen",
        "crafting_screen",
        "furnace_screen",
        "blast_furnace_screen",
        "smoker_screen",
        "anvil_screen",
        "brewing_stand_screen",
        "enchanting_screen",
        "hopper_screen",
        "dispenser_screen",
        "dropper_screen",
        "beacon_screen",
        "grindstone_screen",
        "stonecutter_screen",
        "cartography_screen",
        "loom_screen",
        "smithing_table_screen",
        "horse_screen",
        "trade_screen",
        "crafter_screen",
    ];

    /// The screen kind of `name`. `in_world` is whether a world is joined (the client has a local player).
    ///
    /// The part before ` - ` counts (a progress screen name carries its step after it). `start_screen` is `START`;
    /// an Ore UI page (`ui://...`, the 1.26.70 title screen is `ui://./hbui/index.html`) is `START` while no world
    /// is joined and `OTHER` over a world; `pause_screen` is `PAUSE`; `hud_screen` and `hud_crosshair_screen` are
    /// `HUD`; any name with `progress_screen` is `LOADING`; `toast_screen` is `TOAST`; the container screens are
    /// `CONTAINER`; any other name is `OTHER`.
    pub fn map(name: &str, in_world: bool) -> AmmlScreen {
        let head = name.split(" - ").next().unwrap_or(name);
        match head {
            "start_screen" => AmmlScreen::START,
            "pause_screen" => AmmlScreen::PAUSE,
            "hud_screen" | "hud_crosshair_screen" => AmmlScreen::HUD,
            "toast_screen" => AmmlScreen::TOAST,
            _ if head.starts_with("ui://") => {
                if in_world {
                    AmmlScreen::OTHER
                } else {
                    AmmlScreen::START
                }
            }
            _ if head.contains("progress_screen") => AmmlScreen::LOADING,
            _ if Self::CONTAINERS.contains(&head) => AmmlScreen::CONTAINER,
            _ => AmmlScreen::OTHER,
        }
    }
}

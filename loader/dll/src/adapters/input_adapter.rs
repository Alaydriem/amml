use std::sync::Arc;

use amml_abi::{AmmlEvent, GamepadPoll, KeyInput, LookDelta, MouseInput, PlayerMoveInput};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{
    EightArgMouseFeed, GamepadAdapter, HidKeyFeed, KeyDetours, KeyFeed, LoaderState, LookAdapter, LookMoveDetours,
    MouseDetours, MouseFeed, MoveAdapter, MoveLayout, ProcessorKeyFeed, SessionHook, SevenArgMouseFeed, TurnFeed,
};

// The outcome of one shape install: `Err(Some(refused))` names the refused entries; `Err(None)` when the
// feed already exists (a second install).
type ShapeInstall = Result<(), Option<Vec<&'static str>>>;

/// Installs the input adapters by the catalog shape whose entries resolved, and binds their feeds to the
/// input service.
pub struct InputAdapter;

impl InputAdapter {
    const KEY_INJECT: &'static str = "input.key_inject";
    const MOUSE_INJECT: &'static str = "input.mouse_inject";
    const TURN: &'static str = "input.turn";
    const MOVE: &'static str = "input.move";
    const HID: &'static str = "hid";
    const KEY_PROCESSOR: &'static str = "key_processor";
    const SEVEN_ARG_FEED: &'static str = "seven_arg_feed";
    const EIGHT_ARG_FEED: &'static str = "eight_arg_feed";
    const MOUSE_INSTANCE: &'static str = "MouseDevice.instance";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the key functions of the shape that resolved (`hid` or `key_processor`), the mouse feed of
    /// the shape that resolved (`seven_arg_feed` or `eight_arg_feed`), `LocalPlayer._applyTurnDelta`,
    /// `ClientInputUpdateSystem.extractRawHIDInput` and `GameControllerHandler_GameCore.refresh`, binds
    /// their feeds, offers `input.key`, `input.key_inject`, `input.mouse`, `input.mouse_inject`,
    /// `input.look_delta`, `input.turn`, `input.player_move`, `input.move` and `input.gamepad_poll`, and
    /// logs one `event offered` / `event not available` or `area adapter` / `area not available` line per
    /// row. Returns those rows. A hook that is refused leaves the rows of its kind not available and names
    /// its entry as missing; a move layout that fails its bounds check leaves the move rows not available.
    /// Works once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let catalog = AdapterCatalog::availability(&resolved);
        let kind = |ids: &[&str]| -> Vec<Availability> {
            catalog.iter().filter(|row| ids.contains(&row.id)).cloned().collect()
        };
        let mut rows = Self::install_kind(state, kind(&[KeyInput::ID, Self::KEY_INJECT]), Self::install_keys);
        rows.extend(Self::install_kind(state, kind(&[MouseInput::ID, Self::MOUSE_INJECT]), Self::install_mouse));
        rows.extend(Self::install_kind(state, kind(&[LookDelta::ID, Self::TURN]), Self::install_look));
        rows.extend(Self::install_kind(state, kind(&[PlayerMoveInput::ID, Self::MOVE]), Self::install_move));
        rows.extend(Self::install_kind(state, kind(&[GamepadPoll::ID]), Self::install_gamepad));
        rows
    }

    // Installs the shape of `rows` with `install`, then offers and logs each row.
    fn install_kind(
        state: &'static LoaderState,
        mut rows: Vec<Availability>,
        install: fn(&'static LoaderState, &str) -> ShapeInstall,
    ) -> Vec<Availability> {
        let refused = match rows.iter().find_map(|row| row.shape) {
            Some(shape) => install(state, shape),
            None => Ok(()),
        };
        let refused = match refused {
            Ok(()) => Vec::new(),
            Err(None) => return rows,
            Err(Some(refused)) => refused,
        };
        for row in &mut rows {
            if row.shape.is_some() && !refused.is_empty() {
                row.shape = None;
                row.missing.clone_from(&refused);
            }
            Self::offer(state, row);
        }
        rows
    }

    fn install_keys(state: &'static LoaderState, shape: &str) -> ShapeInstall {
        match shape {
            Self::HID => Self::install_hid(state),
            Self::KEY_PROCESSOR => Self::install_processor(state),
            _ => Ok(()),
        }
    }

    // Hooks the three HID functions and the optional key reset, and binds the feed.
    fn install_hid(state: &'static LoaderState) -> ShapeInstall {
        let feed = HidKeyFeed::create(state).ok_or(None)?;
        Self::hook_all(state, &KeyDetours::hid_hooks())?;
        Self::install_reset(state);
        feed.bind();
        state.input().bind_keys(feed as Arc<dyn KeyFeed>);
        Ok(())
    }

    // Hooks the key reset when its entry resolved. Without it real keys still work, but a key held at a
    // focus loss is released only from the loader's window procedure.
    fn install_reset(state: &'static LoaderState) {
        let entry = KeyDetours::RESET_KEYS;
        let resolved = state.table().get(&EntryId(entry.to_string())).is_some();
        if resolved && KeyDetours::reset_hook().install(state, Self::PRIORITY) {
            curia::info!("key reset hook installed", { entry: entry });
        } else {
            curia::info!("key reset hook not installed: held keys are released on a window focus loss only", {
                entry: entry,
            });
        }
    }

    // Hooks `_feedKeyState` and binds the feed, as `install_hid`.
    fn install_processor(state: &'static LoaderState) -> ShapeInstall {
        let feed = ProcessorKeyFeed::create(state).ok_or(None)?;
        Self::hook_all(state, &[KeyDetours::processor_hook()])?;
        feed.bind();
        state.input().bind_keys(feed as Arc<dyn KeyFeed>);
        Ok(())
    }

    // Hooks the mouse feed of `shape` and binds its feed to the `MouseDevice.instance` device. The shape's
    // entries resolved, so a device address that does not resolve names its entry as refused.
    fn install_mouse(state: &'static LoaderState, shape: &str) -> ShapeInstall {
        let Some(instance) = state.resolve(Self::MOUSE_INSTANCE).ok().and_then(|value| usize::try_from(value).ok())
        else {
            return Err(Some(vec![Self::MOUSE_INSTANCE]));
        };
        match shape {
            Self::SEVEN_ARG_FEED => {
                let feed = SevenArgMouseFeed::create(state, instance).ok_or(None)?;
                Self::hook_all(state, &[MouseDetours::seven_arg_hook()])?;
                state.input().bind_mouse(feed as Arc<dyn MouseFeed>);
            }
            Self::EIGHT_ARG_FEED => {
                let feed = EightArgMouseFeed::create(state, instance).ok_or(None)?;
                Self::hook_all(state, &[MouseDetours::eight_arg_hook()])?;
                state.input().bind_mouse(feed as Arc<dyn MouseFeed>);
            }
            _ => {}
        }
        Ok(())
    }

    // Hooks `_applyTurnDelta` and binds the look adapter as the turn feed.
    fn install_look(state: &'static LoaderState, _shape: &str) -> ShapeInstall {
        let feed = LookAdapter::create(state).ok_or(None)?;
        Self::hook_all(state, &[LookMoveDetours::turn_hook()])?;
        state.input().bind_turn(feed as Arc<dyn TurnFeed>);
        Ok(())
    }

    // Reads the move layout, hooks `extractRawHIDInput` and marks the move point bound. A layout that fails
    // its bounds check hooks nothing.
    fn install_move(state: &'static LoaderState, _shape: &str) -> ShapeInstall {
        let Some(layout) = MoveLayout::from_entries(|name| state.resolve(name).ok()) else {
            curia::warn!("the move layout failed its bounds check: player move is not available");
            return Err(Some(MoveLayout::ENTRIES.to_vec()));
        };
        if !layout.flags_as_expected() {
            curia::warn!("the move flag entries are not the expected values: only the move vectors are used");
        }
        if !MoveAdapter::create(state, layout) {
            return Err(None);
        }
        Self::hook_all(state, &[LookMoveDetours::move_hook()])?;
        state.input().bind_move();
        Ok(())
    }

    // Hooks `refresh`.
    fn install_gamepad(state: &'static LoaderState, _shape: &str) -> ShapeInstall {
        if !GamepadAdapter::create(state) {
            return Err(None);
        }
        Self::hook_all(state, &[LookMoveDetours::gamepad_hook()])
    }

    // Installs every hook; the entries of the refused ones as the error.
    fn hook_all(state: &'static LoaderState, hooks: &[SessionHook]) -> ShapeInstall {
        let refused: Vec<&'static str> =
            hooks.iter().filter(|hook| !hook.install(state, Self::PRIORITY)).map(|hook| hook.entry).collect();
        if refused.is_empty() { Ok(()) } else { Err(Some(refused)) }
    }

    // Offers `row` when it has a shape and logs the outcome.
    fn offer(state: &LoaderState, row: &Availability) {
        match (row.kind, row.shape) {
            (NeedKind::Area, Some(shape)) => {
                state.areas().offer(row.id, row.version);
                curia::info!("area adapter", { area: row.id, shape: shape });
            }
            (NeedKind::Area, None) => {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") })
            }
            (NeedKind::Event, Some(shape)) => {
                state.events().offer(row.id, row.version);
                curia::info!("event offered", { event: row.id, shape: shape });
            }
            (NeedKind::Event, None) => {
                curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
            }
        }
    }
}

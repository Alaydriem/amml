use std::cell::{Cell, RefCell};

use amml_abi::{AmmlImageView, AmmlPlayerListEntry, AmmlStatus, AmmlStr, NETWORK_TEXT_NAME, NETWORK_TEXT_XUID};

use crate::testing::fake_skin::FakeImage;
use crate::testing::{FakeNetworkRun, FakePlayerEntry, FakeSkin};
use crate::{PlayerListAction, Uuid};

/// The network side of a `FakeGame`: the player list or skin event whose dispatch runs now. A network table call answers
/// only inside it, and a skin handle only for the event that gave it, as the loader does. The views the calls give point
/// into the running event, which the fake keeps unchanged until it ends.
#[derive(Default)]
pub(crate) struct FakeNetwork {
    next_serial: Cell<u64>,
    running: RefCell<Option<FakeNetworkRun>>,
}

impl FakeNetwork {
    const INDEX_BITS: u32 = 16;
    // The most animated images the loader gives for one skin.
    const MAX_ANIMATED: usize = 16;

    /// A player list dispatch starts. At most 4096 entries are given.
    pub(crate) fn enter_list(&self, action: PlayerListAction, mut entries: Vec<FakePlayerEntry>) {
        entries.truncate(amml_abi::PlayerListUpdate::MAX_ENTRIES as usize);
        self.start(Some((action, entries)), None);
    }

    /// A skin change dispatch starts.
    pub(crate) fn enter_skin(&self, uuid: Uuid, skin: Option<FakeSkin>) {
        self.start(None, Some((uuid, skin)));
    }

    /// The running dispatch ends; its handles are refused after it.
    pub(crate) fn leave(&self) {
        *self.running.borrow_mut() = None;
    }

    pub(crate) fn entry(&self, index: u32) -> Result<AmmlPlayerListEntry, AmmlStatus> {
        let running = self.running.borrow();
        let running = running.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let (action, entries) = running.list.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let entry = entries.get(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        let add = *action == PlayerListAction::Add;
        let skin = if add && entry.skin.is_some() { Self::handle(running.serial, index as usize) } else { 0 };
        Ok(AmmlPlayerListEntry {
            uuid: entry.uuid.0,
            has_actor: u8::from(add),
            _reserved: [0; 7],
            actor_id: if add { entry.actor_id } else { 0 },
            skin,
        })
    }

    pub(crate) fn entry_text(&self, index: u32, field: u8) -> Result<AmmlStr, AmmlStatus> {
        let running = self.running.borrow();
        let running = running.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let (action, entries) = running.list.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let entry = entries.get(index as usize).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        if *action != PlayerListAction::Add {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        match field {
            NETWORK_TEXT_NAME => Ok(AmmlStr::from_str(&entry.name)),
            NETWORK_TEXT_XUID => Ok(AmmlStr::from_str(&entry.xuid)),
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    pub(crate) fn skin_event(&self) -> Result<([u8; 16], u64), AmmlStatus> {
        let running = self.running.borrow();
        let running = running.as_ref().ok_or(AmmlStatus::REFUSED)?;
        let (uuid, skin) = running.skin_event.as_ref().ok_or(AmmlStatus::REFUSED)?;
        Ok((uuid.0, if skin.is_some() { Self::handle(running.serial, 0) } else { 0 }))
    }

    pub(crate) fn image(&self, handle: u64) -> Result<AmmlImageView, AmmlStatus> {
        self.with_skin(handle, |skin| skin.image.as_ref().map_or(Err(AmmlStatus::NOT_AVAILABLE), Self::view))
    }

    pub(crate) fn animated_count(&self, handle: u64) -> Result<u32, AmmlStatus> {
        self.with_skin(handle, |skin| Ok(skin.animated.len().min(Self::MAX_ANIMATED) as u32))
    }

    pub(crate) fn animated(&self, handle: u64, index: u32) -> Result<(u32, AmmlImageView), AmmlStatus> {
        self.with_skin(handle, |skin| {
            let (kind, image) = skin
                .animated
                .iter()
                .take(Self::MAX_ANIMATED)
                .nth(index as usize)
                .ok_or(AmmlStatus::INVALID_ARGUMENT)?;
            Ok((u32::from(*kind), Self::view(image)?))
        })
    }

    pub(crate) fn persona(&self, handle: u64) -> Result<bool, AmmlStatus> {
        self.with_skin(handle, |skin| Ok(skin.persona))
    }

    pub(crate) fn premium(&self, handle: u64) -> Result<bool, AmmlStatus> {
        self.with_skin(handle, |skin| Ok(skin.premium))
    }

    pub(crate) fn geometry_name(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        self.with_skin(handle, |skin| Self::text(&skin.geometry_name))
    }

    pub(crate) fn geometry_json(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        self.with_skin(handle, |skin| Self::text(&skin.geometry_json))
    }

    pub(crate) fn resource_patch(&self, handle: u64) -> Result<AmmlStr, AmmlStatus> {
        self.with_skin(handle, |skin| Self::text(&skin.resource_patch))
    }

    fn start(
        &self,
        list: Option<(PlayerListAction, Vec<FakePlayerEntry>)>,
        skin_event: Option<(Uuid, Option<FakeSkin>)>,
    ) {
        let serial = self.next_serial.get() + 1;
        self.next_serial.set(serial);
        *self.running.borrow_mut() = Some(FakeNetworkRun { serial, list, skin_event });
    }

    // `serial << 16 | (index + 1)`, as the loader makes it.
    fn handle(serial: u64, index: usize) -> u64 {
        (serial << Self::INDEX_BITS) | (index as u64 + 1)
    }

    // Runs `read` on the skin `handle` names; `REFUSED` for a handle the running event did not give.
    fn with_skin<R>(
        &self,
        handle: u64,
        read: impl FnOnce(&FakeSkin) -> Result<R, AmmlStatus>,
    ) -> Result<R, AmmlStatus> {
        let running = self.running.borrow();
        let running = running.as_ref().ok_or(AmmlStatus::REFUSED)?;
        if handle >> Self::INDEX_BITS != running.serial {
            return Err(AmmlStatus::REFUSED);
        }
        let index = ((handle & ((1 << Self::INDEX_BITS) - 1)) as usize).checked_sub(1).ok_or(AmmlStatus::REFUSED)?;
        let skin = match (&running.list, &running.skin_event) {
            (Some((PlayerListAction::Add, entries)), _) => entries.get(index).and_then(|entry| entry.skin.as_ref()),
            (None, Some((_, skin))) if index == 0 => skin.as_ref(),
            _ => None,
        };
        read(skin.ok_or(AmmlStatus::REFUSED)?)
    }

    fn view(image: &FakeImage) -> Result<AmmlImageView, AmmlStatus> {
        let (width, height, pixels) = image;
        if (*width as usize) * (*height as usize) * 4 != pixels.len() || pixels.is_empty() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(AmmlImageView {
            width: *width,
            height: *height,
            format: crate::PixelFormat::RGBA8,
            _reserved: 0,
            pixels: pixels.as_ptr(),
            len: pixels.len(),
        })
    }

    fn text(text: &Option<String>) -> Result<AmmlStr, AmmlStatus> {
        text.as_deref().map(AmmlStr::from_str).ok_or(AmmlStatus::NOT_AVAILABLE)
    }
}

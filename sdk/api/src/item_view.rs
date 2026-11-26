use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlBlockHandle, AmmlContainerEntry, AmmlInventoryTable, AmmlItemSnapshot, AmmlStatus, AmmlStr};

use crate::{BlockHandle, ContainerContents, ContainerEntry, ItemDrawHandle, ItemSnapshot};

/// A stack of the local player, as a frame handle from `LocalPlayer::held_item` or `Inventory::slot`. It lives no
/// longer than the event that gave its client. Every read goes through the loader, which checks that the stack is
/// still the stack the player holds or a stack it lent in the scope that runs now; a read it refuses gives `None`.
pub struct ItemView<'f> {
    table: AmmlInventoryTable,
    player: *const c_void,
    stack: *const c_void,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl ItemView<'_> {
    // The longest name read in one call; a longer name is read again with its full length.
    const NAME_BUFFER: usize = 128;

    pub(crate) fn new(
        table: AmmlInventoryTable,
        player: *const c_void,
        stack: *const c_void,
        module: &'static str,
    ) -> Self {
        Self { table, player, stack, module, _event: PhantomData }
    }

    /// The light the block this item places gives (0 to 15); `None` for an item that places no block, a refused
    /// read or a loader before `Area::INVENTORY_READ.at(2)`.
    pub fn light_emission(&self) -> Option<u8> {
        self.block_read().filter(|(block, _)| *block != AmmlBlockHandle::NONE).map(|(_, emission)| emission)
    }

    /// The block this item places, as a handle for `LightSource::block`; `None` for an item that places no block,
    /// a refused read or a loader before `Area::INVENTORY_READ.at(2)`.
    pub fn block(&self) -> Option<BlockHandle> {
        self.block_read().and_then(|(block, _)| BlockHandle::from_raw(block))
    }

    fn block_read(&self) -> Option<(AmmlBlockHandle, u8)> {
        let read = self.table.item_block?;
        let mut block = AmmlBlockHandle::NONE;
        let mut emission = 0u8;
        // SAFETY: the loader checks the handles before it reads game memory; the module id lives for the call and
        // both outputs are live locals.
        #[allow(unsafe_code)]
        let status =
            unsafe { read(AmmlStr::from_str(self.module), self.player, self.stack, &mut emission, &mut block) };
        status.is_ok().then_some((block, emission))
    }

    /// The full name of the item type, such as `minecraft:iron_pickaxe`.
    pub fn type_name(&self) -> Option<String> {
        let mut buffer = vec![0u8; Self::NAME_BUFFER];
        let mut len = self.name_into(&mut buffer)?;
        if len > buffer.len() {
            buffer = vec![0u8; len];
            len = self.name_into(&mut buffer)?;
        }
        buffer.truncate(len.min(buffer.len()));
        String::from_utf8(buffer).ok()
    }

    /// The stack size; 0 when the loader refuses the read.
    pub fn count(&self) -> u8 {
        self.read().map_or(0, |raw| raw.count)
    }

    /// Whether the stack holds nothing (a count of 0, or a refused read).
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// The aux value (the variant of the item, such as a dye colour); `None` before `Area::INVENTORY_READ.at(3)` or
    /// on a build that cannot read it.
    pub fn aux(&self) -> Option<i16> {
        self.read().filter(|raw| raw.has_aux != 0).map(|raw| raw.aux)
    }

    /// The most items one stack of this type holds; `None` before `Area::INVENTORY_READ.at(3)` or on a build that
    /// cannot read it.
    pub fn max_count(&self) -> Option<u8> {
        self.read().filter(|raw| raw.has_max_count != 0).map(|raw| raw.max_count)
    }

    /// Whether the stack shows the enchantment glint; `None` before `Area::INVENTORY_READ.at(3)` or on a build that
    /// cannot read it.
    pub fn is_glint(&self) -> Option<bool> {
        self.read().filter(|raw| raw.has_glint != 0).map(|raw| raw.is_glint != 0)
    }

    /// A 64-bit identity of the stack kind: equal for two stacks the game merges (the same type, aux and user data),
    /// whatever their count. `None` for a stack whose user data the loader cannot read, and before
    /// `Area::INVENTORY_READ.at(3)`. The value is the same in every run but may differ between builds that name the
    /// type differently; do not store it across game updates.
    pub fn identity(&self) -> Option<u64> {
        self.read().filter(|raw| raw.has_identity != 0).map(|raw| raw.identity)
    }

    /// The damage the stack has taken.
    pub fn damage(&self) -> Option<i32> {
        self.read().filter(|raw| raw.has_damage != 0).map(|raw| raw.damage)
    }

    /// The most damage the item type takes; 0 for an item with no durability.
    pub fn max_damage(&self) -> Option<i32> {
        self.read().filter(|raw| raw.has_max_damage != 0).map(|raw| raw.max_damage)
    }

    /// A stamp that changes when the stack gets other user data or its user data changes: equal for two reads of one
    /// stack whose tag did not change, so a mod can tell a box swapped for another of the same type and count, which
    /// `identity` (`None` for a stack with user data) cannot. 0 for a stack with no user data. Compare it with the
    /// type, aux and count, which it does not cover. A copy of a stack has another stamp than the stack. `None` for a
    /// refused read, a call off the game thread, a build whose tag layout the loader cannot read, and a loader before
    /// inventory table v6. Game thread.
    pub fn change_stamp(&self) -> Option<u64> {
        let read = self.table.item_change_stamp?;
        let mut stamp = 0u64;
        // SAFETY: the loader checks the player and stack handles before it reads game memory; `stamp` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.player, self.stack, &mut stamp) };
        status.is_ok().then_some(stamp)
    }

    /// Every field in one copy, with the name and the change stamp. A refused read gives an empty snapshot.
    pub fn snapshot(&self) -> ItemSnapshot {
        let raw = self.read().unwrap_or_else(AmmlItemSnapshot::empty);
        ItemSnapshot { change_stamp: self.change_stamp(), ..ItemSnapshot::from_raw(&raw, self.type_name()) }
    }

    /// The contents of this shulker box or bundle: one entry for each stack in the tag list that the game can make, each
    /// with its fields and a handle to draw it with `UiDraw::item`; a bundle in the contents has its own contents. The
    /// loader keeps a game copy of each stack: one call makes at most 1755 (a box of 27 bundles of 64 items reads in full)
    /// and a mod holds at most 4096 draw handles; past either the bundles that do not fit keep no contents, and the call
    /// gives `None` when the top level alone does not fit. `None` also for a stack with no contents list (an empty shulker box has none), a refused read, a call
    /// off the game thread and a loader before `Area::INVENTORY_READ.at(4)`. Game thread; the copies are made in
    /// this call.
    pub fn container_contents(&self) -> Option<ContainerContents> {
        let open = self.table.contents_open?;
        let mut contents = 0u64;
        let mut len = 0u32;
        // SAFETY: the loader checks the handles before it reads game memory; the module id lives for the call and
        // both outputs are live locals.
        #[allow(unsafe_code)]
        let status = unsafe { open(AmmlStr::from_str(self.module), self.player, self.stack, &mut contents, &mut len) };
        if !status.is_ok() || contents == 0 {
            return None;
        }
        self.read_contents(contents)
    }

    // The entries of the list `contents`, then closes it. The list is read until the loader says the index is past the
    // end; any other refusal gives `None`, and the handles made so far drop with the entries.
    fn read_contents(&self, contents: u64) -> Option<ContainerContents> {
        let close = self.table.contents_close?;
        let mut entries = Vec::new();
        let complete = loop {
            let index = u32::try_from(entries.len()).unwrap_or(u32::MAX);
            let mut raw = AmmlContainerEntry::empty();
            // SAFETY: `contents` is an id the loader gave; `raw` is a live local whose `size` covers the v1 struct.
            #[allow(unsafe_code)]
            let status =
                unsafe { (self.table.contents_entry?)(AmmlStr::from_str(self.module), contents, index, &mut raw) };
            if status == AmmlStatus::INVALID_ARGUMENT {
                break true;
            }
            if !status.is_ok() {
                break false;
            }
            // The draw handle first: it owns the loader's copy from here on, whatever happens next.
            let draw = ItemDrawHandle::new(self.table, self.module, raw.draw);
            let name = self.contents_name(contents, index);
            let bundle = if raw.bundle == 0 { None } else { self.read_contents(raw.bundle) };
            entries.push(ContainerEntry {
                slot: raw.slot,
                item: ItemSnapshot::from_raw(&raw.item, name),
                draw,
                bundle,
                bundle_weight: (raw.has_bundle_weight != 0).then_some(raw.bundle_weight),
            });
        };
        // SAFETY: closing an id of the loader is allowed once the entries are read; a closed id does nothing.
        #[allow(unsafe_code)]
        unsafe {
            close(AmmlStr::from_str(self.module), contents);
        }
        complete.then_some(ContainerContents { entries })
    }

    // The type name of entry `index` of the list `contents`.
    fn contents_name(&self, contents: u64, index: u32) -> Option<String> {
        let read = self.table.contents_name?;
        let mut buffer = vec![0u8; Self::NAME_BUFFER];
        let mut len = 0usize;
        for _ in 0..2 {
            // SAFETY: `contents` is an id the loader gave; `buffer` is writable for its length and `len` is a live
            // local.
            #[allow(unsafe_code)]
            let status = unsafe {
                read(AmmlStr::from_str(self.module), contents, index, buffer.as_mut_ptr(), buffer.len(), &mut len)
            };
            if !status.is_ok() {
                return None;
            }
            if len <= buffer.len() {
                buffer.truncate(len);
                return String::from_utf8(buffer).ok();
            }
            buffer = vec![0u8; len];
        }
        None
    }

    fn read(&self) -> Option<AmmlItemSnapshot> {
        let mut raw = AmmlItemSnapshot::empty();
        // SAFETY: the loader checks the player and stack handles before it reads game memory, and `raw` is a live
        // local whose `size` covers the v1 struct.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.item_read)(self.player, self.stack, &mut raw) };
        status.is_ok().then_some(raw)
    }

    // The name into `buffer`; its full length.
    fn name_into(&self, buffer: &mut [u8]) -> Option<usize> {
        let mut len = 0usize;
        // SAFETY: the loader checks the handles before it reads game memory; `buffer` is writable for its length and
        // `len` is a live local.
        #[allow(unsafe_code)]
        let status =
            unsafe { (self.table.item_name)(self.player, self.stack, buffer.as_mut_ptr(), buffer.len(), &mut len) };
        status.is_ok().then_some(len)
    }
}

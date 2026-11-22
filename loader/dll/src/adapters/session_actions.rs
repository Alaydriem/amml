use std::ffi::c_void;
use std::sync::OnceLock;

use amml_abi::{AmmlEvent, AmmlJoinOutcome, AmmlMenu, AmmlStartWorldOutcome, AmmlStatus, JoinServerResult};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::{MsvcHeap, MsvcString, SharedPtr, StaticCallable, StaticCallableVtable};

use crate::{Boundary, ClientAdapter, GameAnswers, JoinBook, LoaderState, OwnerEvent};

static ACTIONS: OnceLock<SessionActions> = OnceLock::new();
static JOIN_VTABLE: StaticCallableVtable = StaticCallableVtable::new(SessionActions::join_answered);
static JOIN_CALLABLE: StaticCallable = StaticCallable::new(&JOIN_VTABLE);

/// `void ClientInstance::requestLeaveGame(bool switchScreen, bool sync)`: rcx the client, dl and r8b.
type LeaveFn = unsafe extern "C" fn(*mut c_void, bool, bool);
/// `World::StartLocalWorldResult World::LocalWorldStarter::startLocalWorld(const World::WorldID&)`: rcx the starter,
/// rdx the id (one std::string at offset 0), the 32-bit result in eax.
type StartLocalWorldFn = unsafe extern "C" fn(*mut c_void, *const MsvcString) -> u32;
/// `void World::ServerWorldJoiner::joinExternalServer(const std::string& host, unsigned short port, const std::string&
/// name, std::function<void(JoinServerWorldResult)> onResult)`: rcx the joiner, rdx the host, r8w the port, r9 the
/// name, the by-value std::function passed as a pointer on the stack.
type JoinExternalFn =
    unsafe extern "C" fn(*mut c_void, *const MsvcString, u16, *const MsvcString, *mut [*mut c_void; 8]);
/// `void ClientInstance.connectByAddress(const std::string& ip, int port)`: rcx the client subobject whose vtable holds
/// it, rdx the ip, r8d the port (26.63 0x65a0552, 70.26 0x8b29880: `mov ebx,r8d; mov rdi,rdx; mov rsi,rcx`).
type ConnectByAddressFn = unsafe extern "C" fn(*mut c_void, *const MsvcString, i32);
/// `bool OreUI::Router::_pushRoute(const std::string& route, RouterPushMode mode)`: rcx the router, rdx the route,
/// r8d the mode, the bool in al.
type PushRouteFn = unsafe extern "C" fn(*mut c_void, *const MsvcString, i32) -> u8;
/// `std::shared_ptr<Packet> MinecraftPackets::createPacket(MinecraftPacketIds)`: rcx the hidden return buffer, edx
/// the id.
type CreatePacketFn = unsafe extern "C" fn(*mut SharedPtr<c_void>, u32) -> *mut SharedPtr<c_void>;
/// `void PacketSender::send(Packet&)`: rcx the sender, rdx the packet.
type SendFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// The session actions of the client (`client.leave_world`, `client.can_join`, `client.start_local_world`,
/// `client.join_server` and `client.join_server_direct` with the `client.join_server_result` event, `client.open_menu`,
/// `client.send_command`), over the primary client.
///
/// Every call runs under the client call rules (`ClientAdapter::check`: gate open, game thread, the primary client)
/// and the offer of its area. A call that needs a world (`leave_world`, `send_command`) gives `REFUSED` without one;
/// a call that needs the menus (`start_local_world`, `join_server`, `open_menu`) gives `REFUSED` in a world. Strings
/// handed to the game are made on the game heap and freed there after the call (`NOT_AVAILABLE` without the heap).
pub struct SessionActions {
    state: &'static LoaderState,
    // Slots of the IClientInstance vtable.
    request_leave: Option<usize>,
    ui_router: Option<usize>,
    packet_sender: Option<usize>,
    // Slots of the MinecraftGame vtable.
    world_system: Option<usize>,
    primary_ready: Option<usize>,
    // Field offsets.
    world_starter_at: Option<usize>,
    network_joiner_at: Option<usize>,
    server_joiner_at: Option<usize>,
    command_at: Option<usize>,
    packet_size: Option<usize>,
    connect_this: Option<usize>,
    // The slot of `send` in the PacketSender vtable.
    send_slot: Option<usize>,
    // Function addresses.
    start_local_world: Option<usize>,
    join_external: Option<usize>,
    push_route: Option<usize>,
    create_packet: Option<usize>,
    connect_by_address: Option<usize>,
    joins: JoinBook,
}

impl SessionActions {
    pub const LEAVE_WORLD: &'static str = "client.leave_world";
    pub const CAN_JOIN: &'static str = "client.can_join";
    pub const START_LOCAL_WORLD: &'static str = "client.start_local_world";
    pub const JOIN_SERVER: &'static str = "client.join_server";
    pub const JOIN_SERVER_DIRECT: &'static str = "client.join_server_direct";
    pub const OPEN_MENU: &'static str = "client.open_menu";
    pub const GO_TO_TITLE: &'static str = "client.go_to_title";
    pub const SEND_COMMAND: &'static str = "client.send_command";
    /// The longest command text `send_command` takes, in bytes.
    pub const MAX_COMMAND: usize = 4096;
    const AREA_VERSION: u32 = 1;
    // `MinecraftPacketIds::CommandRequestPacket`.
    const COMMAND_REQUEST_PACKET: u32 = 0x4D;
    // The Ore UI route of the Play screen's Servers tab, pushed with mode 0.
    const SERVERS_ROUTE: &'static str = "/play/servers";
    // The game's route back to the title with the menu stack emptied: 26.63 builds it in 0x17212a0 (`/main-menu` +
    // `/?` + `shouldFlushStack` + `=` + `true`) and pushes it with `_pushRoute(router, route, 0)` in 0xd8f00
    // (`xor r8d,r8d; call 0xf3c360`); on 70.26 the router took the same text (route seen, via push).
    const TITLE_ROUTE: &'static str = "/main-menu/?shouldFlushStack=true";
    const PUSH_MODE: i32 = 0;

    /// Offers each session action area and the `client.join_server_result` event whose catalog shape resolved, and
    /// logs one line per row. Returns those rows. Works once per process: a second call offers nothing.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_action_row).collect();
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let function = |name: &str| value(name).filter(|address| *address != 0);
        let actions = Self {
            state,
            request_leave: value("ClientInstance.requestLeaveGame"),
            ui_router: value("ClientInstance.getCurrentUIRouter"),
            packet_sender: value("ClientInstance.getPacketSender"),
            world_system: value("MinecraftGame.getWorldSystem"),
            primary_ready: value("MinecraftGame.isPrimaryClientInstanceReady"),
            world_starter_at: value("WorldSystem.mLocalWorldStarter"),
            network_joiner_at: value("WorldSystem.mNetworkWorldJoiner"),
            server_joiner_at: value("NetworkWorldJoiner.mServerWorldJoiner"),
            command_at: value("CommandRequestPacket.mCommand"),
            packet_size: value("CommandRequestPacket.size"),
            connect_this: value("ClientInstance.connectByAddressThis"),
            send_slot: value("PacketSender.send"),
            start_local_world: function("LocalWorldStarter.startLocalWorld"),
            join_external: function("ServerWorldJoiner.joinExternalServer"),
            push_route: function("OreUIRouter.pushRoute"),
            create_packet: function("MinecraftPackets.createPacket"),
            connect_by_address: function("ClientInstance.connectByAddress"),
            joins: JoinBook::new(),
        };
        if ACTIONS.set(actions).is_err() {
            curia::warn!("the session actions are already installed");
            return rows;
        }
        for row in &rows {
            Self::offer(state, row);
        }
        rows
    }

    fn is_action_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Area => [
                Self::LEAVE_WORLD,
                Self::CAN_JOIN,
                Self::START_LOCAL_WORLD,
                Self::JOIN_SERVER,
                Self::JOIN_SERVER_DIRECT,
                Self::OPEN_MENU,
                Self::GO_TO_TITLE,
                Self::SEND_COMMAND,
            ]
            .contains(&row.id),
            NeedKind::Event => row.id == JoinServerResult::ID,
        }
    }

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

    /// The installed actions; `None` before `install`.
    pub fn get() -> Option<&'static SessionActions> {
        ACTIONS.get()
    }

    /// Runs `body` on the installed actions; nothing before `install`.
    pub(crate) fn with(body: impl FnOnce(&SessionActions)) {
        if let Some(actions) = ACTIONS.get() {
            body(actions);
        }
    }

    // The client address when a call in `area` may run: the gate is open (`NOT_READY`), the area is offered
    // (`NOT_AVAILABLE`) and the client rules hold (`ClientAdapter::check`).
    fn enter(&self, client: *const c_void, area: &str) -> Result<usize, AmmlStatus> {
        self.state.events().gate().require_open()?;
        if !self.state.areas().is_available(area, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let adapter = ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        adapter.check(client)
    }

    // Whether a world is joined (the gate epoch is not 0).
    fn in_world(&self) -> bool {
        self.state.events().gate().epoch().0 != 0
    }

    /// Asks the game to leave the joined world: `requestLeaveGame(true, false)`. `REFUSED` while no world is joined.
    pub fn leave_world(&self, client: *const c_void) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::LEAVE_WORLD)?;
        if !self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let function = ClientAdapter::virtual_fn(client, self.request_leave).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `void ClientInstance::requestLeaveGame(bool, bool)`.
        let leave = unsafe { std::mem::transmute::<usize, LeaveFn>(function) };
        curia::info!("leave world asked");
        // SAFETY: called on the live primary client, on the game thread, with the game's own leave arguments.
        unsafe { leave(std::ptr::with_exposed_provenance_mut(client), true, false) };
        Ok(())
    }

    /// Whether the game's primary client is ready for a join: `MinecraftGame::isPrimaryClientInstanceReady`.
    pub fn can_join(&self, client: *const c_void) -> Result<bool, AmmlStatus> {
        let client = self.enter(client, Self::CAN_JOIN)?;
        let game = self.game_of(client)?;
        ClientAdapter::call_bool(game, self.primary_ready)
    }

    /// Starts the local world `level_id` through the game's `LocalWorldStarter`. The game's "not ready" answer is
    /// `NOT_READY`. `REFUSED` in a world; `INVALID_ARGUMENT` for an empty id.
    pub fn start_local_world(
        &self,
        client: *const c_void,
        level_id: &str,
    ) -> Result<AmmlStartWorldOutcome, AmmlStatus> {
        let client = self.enter(client, Self::START_LOCAL_WORLD)?;
        if level_id.is_empty() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let function = self.start_local_world.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let world_system = self.world_system_of(client)?;
        let starter = Self::read_pointer(world_system, self.world_starter_at)?;
        let heap = self.heap()?;
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut id = unsafe { MsvcString::new_in(&heap, level_id) }.map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: the entry is `LocalWorldStarter::startLocalWorld`, with the signature of `StartLocalWorldFn`.
        let start = unsafe { std::mem::transmute::<usize, StartLocalWorldFn>(function) };
        // SAFETY: the starter is live (read from the live world system), on the game thread; the game only reads the
        // id string.
        let code = unsafe { start(std::ptr::with_exposed_provenance_mut(starter), &raw const id) };
        // SAFETY: the string came from `new_in` with this heap and the call did not keep it.
        unsafe { id.release(&heap) };
        curia::info!("local world start asked", { result: code });
        GameAnswers::start_outcome(code)
    }

    /// Joins the server `host`:`port` for the mod at `owner`, shown as `name` (`host` when empty), through the
    /// game's `ServerWorldJoiner::joinExternalServer`. The game answers through the loader's result callable, on any
    /// thread; the answer is raised as `JoinServerResult` to `owner` at the next frame. `REFUSED` in a world or while
    /// a join waits for its answer; `INVALID_ARGUMENT` for an empty host or port 0.
    pub fn join_server(
        &self,
        owner: usize,
        client: *const c_void,
        host: &str,
        port: u16,
        name: &str,
    ) -> Result<u64, AmmlStatus> {
        let client = self.enter(client, Self::JOIN_SERVER)?;
        if host.is_empty() || port == 0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if self.in_world() || self.joins.is_waiting() {
            return Err(AmmlStatus::REFUSED);
        }
        let function = self.join_external.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let member = self.server_joiner_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let world_system = self.world_system_of(client)?;
        let network = Self::read_pointer(world_system, self.network_joiner_at)?;
        let joiner = network.checked_add(member).ok_or(AmmlStatus::FAILED)?;
        let heap = self.heap()?;
        let name = if name.is_empty() { host } else { name };
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut host_text = unsafe { MsvcString::new_in(&heap, host) }.map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: as above.
        let mut name_text = match unsafe { MsvcString::new_in(&heap, name) } {
            Ok(text) => text,
            Err(_) => {
                // SAFETY: the string came from `new_in` with this heap and is not used again.
                unsafe { host_text.release(&heap) };
                return Err(AmmlStatus::FAILED);
            }
        };
        let Some(id) = self.joins.begin(owner) else {
            // SAFETY: both strings came from `new_in` with this heap and are not used again.
            unsafe {
                host_text.release(&heap);
                name_text.release(&heap);
            }
            return Err(AmmlStatus::REFUSED);
        };
        let mut callback = JOIN_CALLABLE.function_block();
        // SAFETY: the entry is `ServerWorldJoiner::joinExternalServer`, with the signature of `JoinExternalFn`.
        let join = unsafe { std::mem::transmute::<usize, JoinExternalFn>(function) };
        curia::info!("server join asked", { join: id, port: port });
        let joiner = std::ptr::with_exposed_provenance_mut::<c_void>(joiner);
        // SAFETY: the joiner is the member of the live NetworkWorldJoiner, on the game thread; the strings are live and
        // the game copies them; the callback block holds the static callable, which the game may copy, move, call and
        // delete on any thread.
        unsafe { join(joiner, &raw const host_text, port, &raw const name_text, &raw mut callback) };
        // SAFETY: both strings came from `new_in` with this heap and are not used again; the call copied them.
        unsafe {
            host_text.release(&heap);
            name_text.release(&heap);
        }
        Ok(id)
    }

    /// Joins the server at the address literal `host` (IPv4 or IPv6), port `port`, for the mod at `owner`, through the
    /// game's own `ClientInstance.connectByAddress`: the game builds the connection info (type `UnknownIP`, transport
    /// 4, the address, the port and its third-party info) and a default `PlayerJoinWorldContext` itself and passes them
    /// to `startExternalNetworkWorld`, the path of the pre-port dejavu replay join. The game's task builds the info again
    /// only for an address literal, so a host name is `INVALID_ARGUMENT`; the host is passed in its canonical text.
    /// `name` is not used (the game passes its own server name). The function is called through the client subobject
    /// at the derived `ClientInstance.connectByAddressThis` (the vptr store of the table that holds it), as the game's
    /// virtual call does: the object itself on 26.63, the base at +0A0h on 70.26.
    /// The game's call has no result, so the join is answered `OTHER` (raised as `JoinServerResult` to `owner` at the
    /// next frame) once the game took it. A `join_server` join that still waits for the game's answer is answered
    /// `OTHER` and replaced. `REFUSED` in a world.
    pub fn join_server_direct(
        &self,
        owner: usize,
        client: *const c_void,
        host: &str,
        port: u16,
        name: &str,
    ) -> Result<u64, AmmlStatus> {
        let client = self.enter(client, Self::JOIN_SERVER_DIRECT)?;
        let address = host.parse::<std::net::IpAddr>().map_err(|_| AmmlStatus::INVALID_ARGUMENT)?;
        if port == 0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let function = self.connect_by_address.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let offset = self.connect_this.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let this = client.checked_add(offset).ok_or(AmmlStatus::FAILED)?;
        let heap = self.heap()?;
        let canonical = address.to_string();
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut host_text = unsafe { MsvcString::new_in(&heap, &canonical) }.map_err(|_| AmmlStatus::FAILED)?;
        if let Some((waiting_owner, waiting)) = self.joins.answer() {
            curia::info!("a waiting server join was replaced by a direct join", { join: waiting });
            let event = OwnerEvent::JoinServerResult { join: waiting, outcome: AmmlJoinOutcome::OTHER };
            self.state.owner_events().push(waiting_owner, event);
        }
        let Some(id) = self.joins.begin(owner) else {
            // SAFETY: the string came from `new_in` with this heap and is not used again.
            unsafe { host_text.release(&heap) };
            return Err(AmmlStatus::REFUSED);
        };
        let module = self.state.module_base() as usize;
        curia::info!("direct server join asked", {
            join: id,
            function: format!("{:#x}", function.wrapping_sub(module)),
            this_offset: format!("{offset:#x}"),
            rdx_host: canonical.as_str(),
            rdx_host_len: canonical.len() as u64,
            r8d_port: port,
            unused_name: name
        });
        // SAFETY: the entry is `ClientInstance.connectByAddress`, with the signature of `ConnectByAddressFn`.
        let connect = unsafe { std::mem::transmute::<usize, ConnectByAddressFn>(function) };
        // SAFETY: `this` is the subobject of the live primary client whose vtable holds the function, on the game
        // thread; the host string is live and the game copies it; the port is the int the function takes.
        unsafe { connect(std::ptr::with_exposed_provenance_mut(this), &raw const host_text, i32::from(port)) };
        // SAFETY: the string came from `new_in` with this heap and is not used again; the call copied it.
        unsafe { host_text.release(&heap) };
        self.answered(0);
        Ok(id)
    }

    /// Opens `menu`: `SERVERS` pushes the Servers route through the client's current Ore UI router. `REFUSED` in a
    /// world or when the router refuses the route; `INVALID_ARGUMENT` for an unknown menu.
    pub fn open_menu(&self, client: *const c_void, menu: AmmlMenu) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::OPEN_MENU)?;
        if menu != AmmlMenu::SERVERS {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let accepted = self.push(client, Self::SERVERS_ROUTE)?;
        curia::info!("menu asked", { menu: menu.0, accepted: accepted });
        if accepted { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    /// Goes back to the title screen: pushes `TITLE_ROUTE` (mode 0) through the client's current Ore UI router, as the
    /// game does. `REFUSED` in a world or when the router refuses the route.
    pub fn go_to_title(&self, client: *const c_void) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::GO_TO_TITLE)?;
        if self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let accepted = self.push(client, Self::TITLE_ROUTE)?;
        curia::info!("title asked", { accepted: accepted });
        if accepted { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    // Pushes `text` (mode `PUSH_MODE`) through the current Ore UI router of `client`; the router's answer.
    fn push(&self, client: usize, text: &str) -> Result<bool, AmmlStatus> {
        let function = self.push_route.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let router = ClientAdapter::call_pointer(client, self.ui_router)?;
        if router == 0 {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let heap = self.heap()?;
        // SAFETY: `heap` is the game's operator new and delete pair.
        let mut route = unsafe { MsvcString::new_in(&heap, text) }.map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: the entry is `OreUI::Router::_pushRoute`, with the signature of `PushRouteFn`.
        let push = unsafe { std::mem::transmute::<usize, PushRouteFn>(function) };
        let router = std::ptr::with_exposed_provenance_mut::<c_void>(router);
        // SAFETY: the router is the live router the client gave, on the game thread; the game copies the route.
        let accepted = unsafe { push(router, &raw const route, Self::PUSH_MODE) };
        // SAFETY: the string came from `new_in` with this heap and the call copied it.
        unsafe { route.release(&heap) };
        Ok(accepted != 0)
    }

    /// Sends `text` as a chat command: a `CommandRequestPacket` from the game's packet factory, the text written over
    /// its empty `mCommand`, sent through the client's packet sender. `REFUSED` without a world or above
    /// `MAX_COMMAND` bytes; `INVALID_ARGUMENT` for an empty text.
    pub fn send_command(&self, client: *const c_void, text: &str) -> Result<(), AmmlStatus> {
        let client = self.enter(client, Self::SEND_COMMAND)?;
        if text.is_empty() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if text.len() > Self::MAX_COMMAND || !self.in_world() {
            return Err(AmmlStatus::REFUSED);
        }
        let factory = self.create_packet.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (Some(command_at), Some(size)) = (self.command_at, self.packet_size) else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        };
        if command_at.checked_add(std::mem::size_of::<MsvcString>()).is_none_or(|end| end > size) {
            curia::warn!("command not sent: the command field is not inside the packet", {
                field: format!("{command_at:#x}"),
                size: format!("{size:#x}")
            });
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let sender = ClientAdapter::call_pointer(client, self.packet_sender)?;
        let send = ClientAdapter::virtual_fn(sender, self.send_slot).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let heap = self.heap()?;
        // SAFETY: `heap` is the game's operator new and delete pair.
        let string = unsafe { MsvcString::new_in(&heap, text) }.map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: an all-zero shared_ptr is the empty one.
        let mut made: SharedPtr<c_void> = unsafe { std::mem::zeroed() };
        // SAFETY: the entry is `MinecraftPackets::createPacket`, with the signature of `CreatePacketFn`.
        let create = unsafe { std::mem::transmute::<usize, CreatePacketFn>(factory) };
        // SAFETY: `made` is 16 writable bytes, on the game thread, and the id is a packet id the game knows.
        unsafe { create(&raw mut made, Self::COMMAND_REQUEST_PACKET) };
        let Some(packet) = made.get().map(|packet| std::ptr::from_ref(packet).cast_mut()) else {
            let mut string = string;
            // SAFETY: the string came from `new_in` with this heap and is not used again.
            unsafe { string.release(&heap) };
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: `packet` is the live CommandRequestPacket the factory made; `command_at` is the offset of its empty
        // `mCommand` (a small string with no block to free), inside the `CommandRequestPacket.size` bytes as checked
        // above. The string moves into the packet, which frees its block in its destructor.
        unsafe { packet.cast::<u8>().add(command_at).cast::<MsvcString>().write_unaligned(string) };
        // SAFETY: the slot holds `void PacketSender::send(Packet&)`.
        let send = unsafe { std::mem::transmute::<usize, SendFn>(send) };
        // SAFETY: the sender and the packet are live, on the game thread; the call does not take the packet.
        unsafe { send(std::ptr::with_exposed_provenance_mut(sender), packet) };
        // SAFETY: `made` holds the one use the factory gave; the last use runs the packet's destructor.
        unsafe { made.release() };
        curia::info!("command sent", { bytes: text.len() });
        Ok(())
    }

    /// After a world is joined: a join that still waits gets its answer now, as `OTHER` (the game went on with it).
    pub(crate) fn world_joined(&self) {
        if let Some((owner, id)) = self.joins.answer() {
            curia::info!("server join answered at the world join", { join: id });
            let event = OwnerEvent::JoinServerResult { join: id, outcome: AmmlJoinOutcome::OTHER };
            self.state.owner_events().push(owner, event);
        }
    }

    /// The game's join answer `code`, from the result callable on any thread: the waiting join's result is queued
    /// for its owner. A second answer for the same join finds none waiting and is dropped.
    pub fn answered(&self, code: i32) {
        let outcome = GameAnswers::join_outcome(code);
        match self.joins.answer() {
            Some((owner, id)) => {
                curia::info!("server join answered", {
                    join: id,
                    result: code,
                    outcome: GameAnswers::join_outcome_name(outcome)
                });
                self.state.owner_events().push(owner, OwnerEvent::JoinServerResult { join: id, outcome });
            }
            None => curia::info!("a server join answer with no waiting join was dropped", { result: code }),
        }
    }

    // `_Do_call` of the join result callable: may run on a game worker thread, so it only reads the code and queues.
    unsafe extern "C" fn join_answered(_this: *mut c_void, result: *const c_void) {
        Boundary::run((), || {
            // SAFETY: the game passes a pointer to the 32-bit result code; null is read as -1.
            let code = if result.is_null() { -1 } else { unsafe { result.cast::<i32>().read() } };
            Self::with(|actions| actions.answered(code));
        });
    }

    // The MinecraftGame of `client`, checked by `ClientAdapter::game_of`.
    fn game_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.game_of(client)
    }

    // The world system of the client's MinecraftGame; null gives `NOT_AVAILABLE`.
    fn world_system_of(&self, client: usize) -> Result<usize, AmmlStatus> {
        let game = self.game_of(client)?;
        match ClientAdapter::call_pointer(game, self.world_system)? {
            0 => Err(AmmlStatus::NOT_AVAILABLE),
            system => Ok(system),
        }
    }

    // The non-null pointer at `offset` in `object`.
    fn read_pointer(object: usize, offset: Option<usize>) -> Result<usize, AmmlStatus> {
        let address = object.checked_add(offset.ok_or(AmmlStatus::NOT_AVAILABLE)?).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `object` is a live game object on the game thread and `offset` is the derived offset of a pointer
        // member inside it, so the qword is readable; members of these objects are 8-aligned.
        let value = unsafe { std::ptr::with_exposed_provenance::<usize>(address).read() };
        if value == 0 { Err(AmmlStatus::NOT_AVAILABLE) } else { Ok(value) }
    }

    fn heap(&self) -> Result<MsvcHeap, AmmlStatus> {
        if !self.state.heap().is_available() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete))
    }
}

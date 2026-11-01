//! The AMML test mod: proves resolve, hook and log in the running game, and reads the player, the dimension and
//! the camera through `amml-game`.

#![deny(unsafe_code)]

pub mod chunk_reading;
pub mod client_update;
pub mod controller_dtor;
pub mod create_view_render_object;
pub mod extract_name_tags;
pub mod font_handle_layout;
pub mod font_repository_layout;
pub mod font_scope;
pub mod layout_fault;
pub mod light_reading;
pub mod name_tag_collection_layout;
pub mod name_tag_entries;
pub mod name_tag_layout;
pub mod nbt_dump;
pub mod peek_entries;
pub mod peek_hover;
pub mod peek_probe;
pub mod peek_report;
pub mod probe;
pub mod probe_report;
pub mod screen_view_render;
pub mod setup_camera;
pub mod slot_hovered;
pub mod slot_unhovered;
pub mod test_mod;
pub mod throttle;
pub mod tick_counter;
pub mod ui_scene_direction;
pub mod world_latch;

pub use chunk_reading::ChunkReading;
pub use font_handle_layout::FontHandleLayout;
pub use font_repository_layout::FontRepositoryLayout;
pub use font_scope::FontScope;
pub use layout_fault::LayoutFault;
pub use light_reading::LightReading;
pub use name_tag_collection_layout::NameTagCollectionLayout;
pub use name_tag_entries::NameTagEntries;
pub use name_tag_layout::NameTagLayout;
pub use nbt_dump::NbtDump;
pub use peek_entries::PeekEntries;
pub use peek_hover::PeekHover;
pub use peek_probe::PeekProbe;
pub use peek_report::PeekReport;
pub use probe::Probe;
pub use probe_report::ProbeReport;
pub use test_mod::TestMod;
pub use throttle::Throttle;
pub use tick_counter::TickCounter;
pub use world_latch::WorldLatch;

use amml_sdk::amml_mod;

/// Calls of `ClientInstance::update` counted by the detour.
pub static TICKS: TickCounter = TickCounter::new(100);

/// The one probe of this DLL; the three detours share it.
pub static PROBE: Probe = Probe::new();

/// The probe of the shulker box preview hooks; the five detours share it.
pub static PEEK: PeekProbe = PeekProbe::new();

amml_mod!(TestMod);

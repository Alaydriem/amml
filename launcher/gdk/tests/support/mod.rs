pub mod fake_deployer;
pub mod fake_exe_copier;
pub mod fake_installer;
pub mod fake_inventory;
pub mod fake_processes;
pub mod fake_server;
pub mod game_folder;
pub mod pe_bytes;

pub use fake_deployer::{FakeCall, FakeDeployer};
pub use fake_exe_copier::FakeExeCopier;
pub use fake_installer::{FakeDeveloperMode, FakeInstaller};
pub use fake_inventory::{FakeInventory, Packages};
pub use fake_processes::FakeProcesses;
pub use fake_server::{FakeServer, Route};
pub use game_folder::GameFolder;
pub use pe_bytes::PeBytes;

pub struct Bodies;

impl Bodies {
    // deterministic bytes whose pattern depends on `seed`
    pub fn make(len: usize, seed: u8) -> Vec<u8> {
        (0..len).map(|i| ((i * 31 + seed as usize) % 251) as u8).collect()
    }
}

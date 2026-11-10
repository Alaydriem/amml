// helpers are shared by many test modules; each module uses a part
#![allow(dead_code)]

pub mod catalogues;
pub mod fake_installer;
pub mod fake_patcher;
pub mod fake_processes;
pub mod fake_server;
pub mod fake_starter;
pub mod fake_window_probe;
pub mod harness;
pub mod instance_folder;
pub mod pe_bytes;
pub mod proof_harness;
pub mod recorded_request;
pub mod registry_tree;
pub mod roots;
pub mod route;
pub mod zips;

pub use catalogues::Catalogues;
pub use fake_installer::FakeInstaller;
pub use fake_patcher::FakePatcher;
pub use fake_processes::FakeProcesses;
pub use fake_server::FakeServer;
pub use fake_starter::FakeStarter;
pub use fake_window_probe::FakeWindowProbe;
pub use harness::Harness;
pub use instance_folder::InstanceFolder;
pub use pe_bytes::PeBytes;
pub use proof_harness::ProofHarness;
pub use recorded_request::RecordedRequest;
pub use registry_tree::RegistryTree;
pub use roots::Roots;
pub use route::Route;
pub use zips::Zips;

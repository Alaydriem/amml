use crate::support::{FakeInventory, FakeProcesses, Packages};
use amml_gdk::{Channel, DiskSpace, SpaceNeed, StoreCheck, StoreState};
use std::path::PathBuf;

const SMALL: SpaceNeed = SpaceNeed { work_drive: 1, game_drive: 0 };

struct Rig {
    inventory: FakeInventory,
    processes: FakeProcesses,
    root: tempfile::TempDir,
}

impl Rig {
    fn new() -> Rig {
        let rig = Rig {
            inventory: FakeInventory::new(),
            processes: FakeProcesses::new(),
            root: tempfile::tempdir().unwrap(),
        };
        rig.inventory.set(Packages::gaming_services(rig.root.path()));
        rig.inventory.set(Packages::bedrock("6005", rig.root.path()));
        rig
    }

    fn check(&self, channel: Channel, need: SpaceNeed, block_running: bool) -> StoreState {
        let check = StoreCheck::new(self.inventory.clone(), self.processes.clone(), DiskSpace::new());
        check.check(channel, need, self.root.path(), block_running).unwrap()
    }
}

#[test]
fn ready_when_bedrock_and_gaming_services_present() {
    let rig = Rig::new();
    match rig.check(Channel::Release, SMALL, true) {
        StoreState::Ready(package) => {
            assert_eq!(package.full_name, "Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe")
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn release_missing_is_bedrock_missing() {
    let rig = Rig::new();
    rig.inventory.clear(Packages::RELEASE);
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::BedrockMissing);
}

#[test]
fn preview_needs_preview_app() {
    let rig = Rig::new();
    assert_eq!(rig.check(Channel::Preview, SMALL, true), StoreState::PreviewMissing);
}

#[test]
fn gaming_services_missing_is_named() {
    let rig = Rig::new();
    rig.inventory.clear(Packages::GAMING_SERVICES);
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::GamingServicesMissing);
}

#[test]
fn unhealthy_package_is_named() {
    let rig = Rig::new();
    let mut package = Packages::bedrock("6005", rig.root.path());
    package.status_ok = false;
    rig.inventory.set(package.clone());
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::NotHealthy { full_name: package.full_name });
}

#[test]
fn sideloaded_package_is_refused() {
    let rig = Rig::new();
    let mut package = Packages::bedrock("6005", rig.root.path());
    package.store_signed = false;
    rig.inventory.set(package.clone());
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::NotStoreSigned { full_name: package.full_name });
}

#[test]
fn registered_game_running_blocks() {
    let rig = Rig::new();
    let package = Packages::bedrock("6005", rig.root.path());
    rig.processes.start(42, package.content_location.join("Minecraft.Windows.exe"));
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::MinecraftRunning { pids: vec![42] });
}

#[test]
fn a_sibling_folder_with_the_same_prefix_is_not_the_registered_game() {
    let rig = Rig::new();
    let package = Packages::bedrock("6005", rig.root.path());
    let mut sibling = package.content_location.into_os_string();
    sibling.push("-copy");
    rig.processes.start(7, PathBuf::from(sibling).join("Minecraft.Windows.exe"));
    assert!(matches!(rig.check(Channel::Release, SMALL, true), StoreState::Ready(_)));
}

#[test]
fn loose_instance_running_does_not_block() {
    let rig = Rig::new();
    let loose = rig.root.path().join("amml").join("instances").join("x").join("Minecraft.Windows.exe");
    rig.processes.start(99, loose);
    assert!(matches!(rig.check(Channel::Release, SMALL, true), StoreState::Ready(_)));
}

#[test]
fn running_game_ignored_when_not_blocking() {
    let rig = Rig::new();
    let package = Packages::bedrock("6005", rig.root.path());
    rig.processes.start(42, package.install_location.join("Minecraft.Windows.exe"));
    assert!(matches!(rig.check(Channel::Release, SMALL, false), StoreState::Ready(_)));
}

#[test]
fn huge_install_is_not_enough_space() {
    let rig = Rig::new();
    let need = SpaceNeed { work_drive: u64::MAX / 4, game_drive: 0 };
    match rig.check(Channel::Release, need, true) {
        StoreState::NotEnoughSpace { needed, free, .. } => {
            assert_eq!(needed, u64::MAX / 4);
            assert!(free < needed);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn game_drive_space_is_checked_when_needed() {
    let rig = Rig::new();
    let need = SpaceNeed { work_drive: 1, game_drive: u64::MAX / 4 };
    assert!(matches!(rig.check(Channel::Release, need, true), StoreState::NotEnoughSpace { .. }));
}

#[test]
fn space_need_counts_only_missing_downloads() {
    let need = SpaceNeed::for_install(100, 200, true, false);
    assert_eq!(need.work_drive, 200 + 100 + 1024 * 1024 * 1024);
    assert_eq!(need.game_drive, 100);
}

#[test]
fn a_game_process_with_an_unreadable_path_counts_as_running() {
    let rig = Rig::new();
    rig.processes.start(77, PathBuf::new());
    assert_eq!(rig.check(Channel::Release, SMALL, true), StoreState::MinecraftRunning { pids: vec![77] });
    assert!(matches!(rig.check(Channel::Release, SMALL, false), StoreState::Ready(_)));
}

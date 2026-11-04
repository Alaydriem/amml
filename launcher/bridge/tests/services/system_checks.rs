use crate::support::{FakeInventory, Samples};
use amml_launcher_bridge::{CheckActionView, CheckIdView, CheckStateView, SystemChecks};
use std::path::Path;

const BEDROCK: &str = "Microsoft.MinecraftUWP_8wekyb3d8bbwe";
const PREVIEW: &str = "Microsoft.MinecraftWindowsBeta_8wekyb3d8bbwe";
const SERVICES: &str = "Microsoft.GamingServices_8wekyb3d8bbwe";

fn everything() -> FakeInventory {
    FakeInventory::new(vec![
        (BEDROCK, Samples::package(BEDROCK, "1.26.60.5")),
        (SERVICES, Samples::package(SERVICES, "1.2.3.4")),
    ])
}

#[test]
fn the_checks_come_in_the_order_of_the_screen() {
    let checks = SystemChecks::run(&everything(), true, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    let ids: Vec<CheckIdView> = checks.iter().map(|check| check.id).collect();
    assert_eq!(
        ids,
        vec![
            CheckIdView::Store,
            CheckIdView::GamingServices,
            CheckIdView::Gameinput,
            CheckIdView::Preview,
            CheckIdView::Disk
        ]
    );
    assert_eq!(checks[0].state, CheckStateView::Ok);
    assert_eq!(checks[0].version.as_deref(), Some("1.26.60.5"));
    assert_eq!(checks[1].state, CheckStateView::Ok);
    assert_eq!(checks[3].state, CheckStateView::Missing);
    assert_eq!(checks[3].action, Some(CheckActionView::OpenStore));
}

#[test]
fn low_disk_space_is_a_warning_with_the_free_bytes() {
    let low = SystemChecks::run(&everything(), true, 3_400_000_000, Path::new("C:\\amml\\data")).unwrap();
    let disk = &low[4];
    assert_eq!(disk.state, CheckStateView::Warn);
    assert_eq!(disk.free_bytes, Some(3_400_000_000));
    assert_eq!(disk.drive.as_deref(), Some("C:\\"));

    let enough = SystemChecks::run(&everything(), true, SystemChecks::VERSION_SPACE, Path::new("C:\\amml")).unwrap();
    assert_eq!(enough[4].state, CheckStateView::Ok);
}

#[test]
fn a_missing_game_input_offers_an_elevated_install() {
    let checks = SystemChecks::run(&everything(), false, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    let input = &checks[2];
    assert_eq!(input.state, CheckStateView::Missing);
    assert_eq!(input.action, Some(CheckActionView::Install));
    assert!(input.elevated);

    let present = SystemChecks::run(&everything(), true, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    assert_eq!(present[2].action, None);
    assert!(!present[2].elevated);
}

#[test]
fn a_missing_store_game_or_gaming_services_offers_the_store() {
    let nothing = FakeInventory::new(vec![]);
    let checks = SystemChecks::run(&nothing, true, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    for index in [0, 1] {
        assert_eq!(checks[index].state, CheckStateView::Missing);
        assert_eq!(checks[index].action, Some(CheckActionView::OpenStore));
        assert_eq!(checks[index].package, None);
    }
}

#[test]
fn a_package_that_is_not_healthy_is_a_warning() {
    let mut broken = Samples::package(BEDROCK, "1.26.60.5");
    broken.status_ok = false;
    let inventory = FakeInventory::new(vec![(BEDROCK, broken)]);
    let checks = SystemChecks::run(&inventory, true, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    assert_eq!(checks[0].state, CheckStateView::Warn);
}

#[test]
fn an_installed_preview_app_is_ok() {
    let inventory = FakeInventory::new(vec![(PREVIEW, Samples::package(PREVIEW, "1.26.70.20"))]);
    let checks = SystemChecks::run(&inventory, true, 212_000_000_000, Path::new("C:\\amml")).unwrap();
    assert_eq!(checks[3].state, CheckStateView::Ok);
    assert_eq!(checks[3].version.as_deref(), Some("1.26.70.20"));
}

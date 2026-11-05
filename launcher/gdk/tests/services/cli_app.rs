use crate::support::{FakeDeveloperMode, FakeInstaller, FakeInventory, GameFolder, Packages, PeBytes};
use amml_gdk::{
    AmmlPaths, CatalogueReader, CliApp, CliCommand, CliExit, GameVersion, GdkError, InstallOutcome, StoreState,
    VersionFolder,
};
use std::path::PathBuf;

const CATALOGUE: &str = r#"{
  "release": {
    "1.26.60.5": ["https://a.example/Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe.msixvc"],
    "1.26.52.3": ["https://a.example/Microsoft.MinecraftUWP_1.26.5203.0_x64__8wekyb3d8bbwe.msixvc"],
    "1.21.120.4": ["https://a.example/Microsoft.MinecraftUWP_1.21.12004.0_x64__8wekyb3d8bbwe.msixvc"]
  },
  "preview": {}
}"#;

struct Rig {
    root: tempfile::TempDir,
    installer: FakeInstaller,
    inventory: FakeInventory,
    developer_mode: FakeDeveloperMode,
    app: CliApp<FakeInstaller, FakeInventory, FakeDeveloperMode>,
}

struct Run {
    exit: CliExit,
    text: String,
}

impl Rig {
    fn new() -> Rig {
        let root = tempfile::tempdir().unwrap();
        let installer = FakeInstaller::new();
        let inventory = FakeInventory::new();
        let developer_mode = FakeDeveloperMode::new();
        let app = CliApp::new(
            installer.clone(),
            inventory.clone(),
            developer_mode.clone(),
            AmmlPaths::new(root.path().join("amml")),
            CatalogueReader::parse(CATALOGUE).unwrap(),
        );
        Rig { root, installer, inventory, developer_mode, app }
    }

    fn run(&self, command: CliCommand, input: &str) -> Run {
        let mut output = Vec::new();
        let exit = self.app.run(&command, &mut input.as_bytes(), &mut output);
        Run { exit, text: String::from_utf8(output).unwrap() }
    }

    fn install_command(version: &str) -> CliCommand {
        CliCommand::Install { version: version.to_string(), preview: false }
    }

    fn outcome() -> InstallOutcome {
        InstallOutcome {
            version: VersionFolder {
                version: GameVersion::parse("1.26.52.3").unwrap(),
                path: PathBuf::from("versions/1.26.52.03"),
                files: 12178,
                bytes: 1,
            },
            path_used: "install-restore",
            final_full_name: Packages::full_name("6005"),
            store_updated_meanwhile: false,
        }
    }

    fn exe_copy_failure() -> GdkError {
        GdkError::ExeCopyFailed { method: "package-context", detail: "the cmdlet is missing".to_string() }
    }
}

#[test]
fn install_restore_failed_exits_3_and_prints_fix() {
    let rig = Rig::new();
    rig.installer.queue(Err(GdkError::RestoreFailed {
        original: Packages::full_name("6005"),
        package: PathBuf::from(r"C:\Users\x\amml\launcher\downloads\a.msixvc"),
        cause: "the add failed".to_string(),
    }));

    let run = rig.run(Rig::install_command("1.26.52.3"), "");

    assert_eq!(run.exit, CliExit::RestoreFailed);
    assert_eq!(run.exit.code(), 3);
    let fix = r"Add-AppxPackage -ForceUpdateFromAnyVersion -Path 'C:\Users\x\amml\launcher\downloads\a.msixvc'";
    assert!(run.text.contains(fix), "{}", run.text);
}

#[test]
fn not_ready_exits_2_and_names_state() {
    let rig = Rig::new();
    rig.installer.queue(Err(GdkError::NotReady(Box::new(StoreState::MinecraftRunning { pids: vec![42] }))));

    let run = rig.run(Rig::install_command("1.26.52.3"), "");

    assert_eq!(run.exit.code(), 2);
    assert!(run.text.contains("Minecraft is running"), "{}", run.text);
    assert!(run.text.contains("42"));
}

#[test]
fn developer_mode_prompt_only_on_exe_copy_failure() {
    let rig = Rig::new();
    rig.installer.queue(Err(Rig::exe_copy_failure()));

    let declined = rig.run(Rig::install_command("1.26.52.3"), "n\n");

    assert_eq!(declined.exit.code(), 1);
    assert!(declined.text.contains("Turn on Developer Mode (UAC prompt) and retry? [y/N]"));
    assert_eq!(rig.developer_mode.enable_calls(), 0);
    assert_eq!(rig.installer.install_calls(), 1);

    rig.installer.queue(Err(GdkError::Busy));
    let other = rig.run(Rig::install_command("1.26.52.3"), "y\n");

    assert_eq!(other.exit.code(), 1);
    assert!(!other.text.contains("Developer Mode"));
    assert_eq!(rig.developer_mode.enable_calls(), 0);
}

#[test]
fn developer_mode_yes_turns_it_on_and_retries_once() {
    let rig = Rig::new();
    rig.installer.queue(Err(Rig::exe_copy_failure()));
    rig.installer.queue(Ok(Rig::outcome()));

    let run = rig.run(Rig::install_command("1.26.52.3"), "y\n");

    assert_eq!(run.exit, CliExit::Ok);
    assert_eq!(rig.developer_mode.enable_calls(), 1);
    assert_eq!(rig.installer.install_calls(), 2);
    assert!(run.text.contains("Installed 1.26.52.3"));
}

#[test]
fn a_second_exe_copy_failure_does_not_ask_again() {
    let rig = Rig::new();
    rig.installer.queue(Err(Rig::exe_copy_failure()));
    rig.installer.queue(Err(Rig::exe_copy_failure()));

    let run = rig.run(Rig::install_command("1.26.52.3"), "y\ny\n");

    assert_eq!(run.exit.code(), 1);
    assert_eq!(rig.developer_mode.enable_calls(), 1);
    assert_eq!(rig.installer.install_calls(), 2);
}

#[test]
fn a_bad_version_is_an_error_before_any_install() {
    let rig = Rig::new();
    let run = rig.run(Rig::install_command("latest"), "");
    assert_eq!(run.exit.code(), 1);
    assert_eq!(rig.installer.install_calls(), 0);
}

#[test]
fn list_marks_installed_and_stored_versions() {
    let rig = Rig::new();
    rig.inventory.set(Packages::bedrock("6005", rig.root.path()));
    GameFolder::create(&rig.root.path().join("amml").join("versions").join("1.26.52.03"));

    let run = rig.run(CliCommand::List { preview: false }, "");

    let lines: Vec<&str> = run.text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "1.26.60.5  (installed in the Store)");
    assert_eq!(lines[1], "1.26.52.3  (in the version store)");
    assert_eq!(lines[2], "1.21.120.4");
}

#[test]
fn seed_reads_the_version_from_version_json() {
    let rig = Rig::new();
    let instance = rig.root.path().join("instance");
    GameFolder::create(&instance);
    std::fs::write(instance.join("Minecraft.Windows.exe.bak"), PeBytes::x64(8192)).unwrap();
    std::fs::write(instance.join("version.json"), r#"{"gameVersion": "1.26.60.05"}"#).unwrap();

    let run = rig.run(CliCommand::Seed { instance_dir: instance }, "");

    assert_eq!(run.exit, CliExit::Ok, "{}", run.text);
    assert!(rig.root.path().join("amml").join("versions").join("1.26.60.05").join("MicrosoftGame.Config").is_file());
}

#[test]
fn recover_reports_the_restored_package_or_nothing() {
    let rig = Rig::new();
    rig.installer.queue_recover(Ok(Some(Packages::full_name("6005"))));
    rig.installer.queue_recover(Ok(None));

    let restored = rig.run(CliCommand::Recover, "");
    let nothing = rig.run(CliCommand::Recover, "");

    assert!(restored.text.contains(&Packages::full_name("6005")));
    assert!(nothing.text.contains("Nothing to recover"));
}

#[test]
fn status_names_the_registered_package_and_a_waiting_journal() {
    let rig = Rig::new();
    rig.inventory.set(Packages::bedrock("6005", rig.root.path()));
    let journal = rig.root.path().join("amml").join("launcher").join("install-journal.hcl");
    std::fs::create_dir_all(journal.parent().unwrap()).unwrap();
    std::fs::write(&journal, "x").unwrap();

    let run = rig.run(CliCommand::Status, "");

    assert!(run.text.contains(&format!("Release: {} (Ok)", Packages::full_name("6005"))), "{}", run.text);
    assert!(run.text.contains("Preview: not installed"));
    assert!(run.text.contains("Developer Mode: off"));
    assert!(run.text.contains("Install journal: present"));
}

#[test]
fn the_fix_line_doubles_every_kind_of_single_quote_in_the_path() {
    let rig = Rig::new();
    let path = "C:\x5CUsers\x5CO\u{2019}Brien\x5Cit's.msixvc".to_string();
    rig.installer.queue(Err(GdkError::RestoreFailed {
        original: Packages::full_name("6005"),
        package: PathBuf::from(&path),
        cause: "x".to_string(),
    }));

    let run = rig.run(Rig::install_command("1.26.52.3"), "");

    let quoted = path.replace('\u{2019}', "\u{2019}\u{2019}").replace('\'', "''");
    assert!(run.text.contains(&format!("-Path '{quoted}'")), "{}", run.text);
}

#[test]
fn an_unreadable_journal_exits_3_and_lists_the_cached_packages() {
    let rig = Rig::new();
    let downloads = rig.root.path().join("amml").join("launcher").join("downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    std::fs::write(downloads.join("a.msixvc"), b"x").unwrap();
    std::fs::write(downloads.join("b.msixvc.part"), b"x").unwrap();
    rig.installer.queue(Err(GdkError::BadJournal("the file is empty".to_string())));

    let run = rig.run(Rig::install_command("1.26.52.3"), "");

    assert_eq!(run.exit.code(), 3);
    assert!(run.text.contains("install-journal.hcl"), "{}", run.text);
    assert!(run.text.contains("a.msixvc'"));
    assert!(!run.text.contains("b.msixvc"));
    assert!(run.text.contains("reinstall Minecraft from the Microsoft Store"));
}

#[test]
fn instance_command_writes_version_json() {
    let rig = Rig::new();
    let seeded = rig.root.path().join("seeded");
    GameFolder::create(&seeded);
    std::fs::write(seeded.join("Minecraft.Windows.exe.bak"), PeBytes::x64(8192)).unwrap();
    std::fs::write(seeded.join("version.json"), r#"{"gameVersion": "1.26.60.05"}"#).unwrap();
    assert_eq!(rig.run(CliCommand::Seed { instance_dir: seeded }, "").exit, CliExit::Ok);

    let run = rig.run(CliCommand::Instance { version: "1.26.60.5".to_string(), name: "test-x".to_string() }, "");

    assert_eq!(run.exit, CliExit::Ok, "{}", run.text);
    let path = rig.root.path().join("amml").join("instances").join("test-x").join("version.json");
    let document = amml_gdk::VersionJson::read(&path).unwrap();
    assert_eq!(document.game_version().unwrap(), GameVersion::parse("1.26.60.5").unwrap());
    assert_eq!(document.value("name"), Some(&serde_json::json!("test-x")));
    assert_eq!(document.value("launcher"), None);
}

#[test]
fn instance_command_writes_no_version_json_when_create_fails() {
    let rig = Rig::new();
    let existing = rig.root.path().join("amml").join("instances").join("taken");
    std::fs::create_dir_all(&existing).unwrap();

    let run = rig.run(CliCommand::Instance { version: "1.26.60.5".to_string(), name: "taken".to_string() }, "");

    assert_eq!(run.exit, CliExit::Error, "{}", run.text);
    assert!(!existing.join("version.json").exists());
}

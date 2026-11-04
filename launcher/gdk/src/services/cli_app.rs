use super::{DeveloperModeControl, Installer, InstanceService, PackageInventory, VersionStore};
use crate::GdkError;
use crate::structs::{
    AmmlPaths, Catalogue, Channel, CliCommand, CliExit, GameVersion, InstallOutcome, InstallRequest, InstallStep,
    PowerShellText,
};
use std::fmt::Display;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

// The `amml-gdk` command line. It reads and writes through the streams it is given, so a test drives it with text.
pub struct CliApp<N, I, M> {
    installer: N,
    inventory: I,
    developer_mode: M,
    instances: InstanceService,
    store: VersionStore,
    paths: AmmlPaths,
    catalogue: Catalogue,
}

impl<N: Installer, I: PackageInventory, M: DeveloperModeControl> CliApp<N, I, M> {
    pub fn new(
        installer: N,
        inventory: I,
        developer_mode: M,
        paths: AmmlPaths,
        catalogue: Catalogue,
    ) -> CliApp<N, I, M> {
        CliApp {
            installer,
            inventory,
            developer_mode,
            instances: InstanceService::new(paths.clone(), VersionStore::new(paths.clone())),
            store: VersionStore::new(paths.clone()),
            paths,
            catalogue,
        }
    }

    pub fn run(&self, command: &CliCommand, input: &mut impl BufRead, output: &mut impl Write) -> CliExit {
        let result = match command {
            CliCommand::List { preview } => self.list(Self::channel(*preview), output),
            CliCommand::Install { version, preview } => self.install(version, Self::channel(*preview), input, output),
            CliCommand::Instance { version, name } => self.instance(version, name, output),
            CliCommand::Seed { instance_dir } => self.seed(instance_dir, output),
            CliCommand::Recover => self.recover(output),
            CliCommand::Status => self.status(output),
        };
        match result {
            Ok(()) => CliExit::Ok,
            Err(error) => self.report(&error, output),
        }
    }

    fn channel(preview: bool) -> Channel {
        if preview { Channel::Preview } else { Channel::Release }
    }

    fn line(output: &mut impl Write, text: impl Display) {
        let _ = writeln!(output, "{text}");
    }

    fn report(&self, error: &GdkError, output: &mut impl Write) -> CliExit {
        match error {
            GdkError::NotReady(state) => {
                Self::line(output, format!("Not ready: {}", state.describe()));
                CliExit::NotReady
            }
            GdkError::RestoreFailed { original, package, cause } => {
                Self::line(output, format!("The installed version {original} was not restored: {cause}"));
                Self::line(output, "Put it back by running this in Windows PowerShell:");
                Self::line(output, Self::add_package_line(package));
                Self::line(output, "or reinstall Minecraft from the Microsoft Store.");
                CliExit::RestoreFailed
            }
            GdkError::BadJournal(cause) => {
                Self::line(
                    output,
                    format!("The install journal {} cannot be read: {cause}", self.paths.journal().display()),
                );
                Self::line(
                    output,
                    "A Minecraft install may have been stopped half way. To put the Store version back,",
                );
                Self::line(output, "run one of these in Windows PowerShell, for the Minecraft version you had:");
                let cached = self.cached_packages();
                if cached.is_empty() {
                    Self::line(output, format!("  (no package is cached in {})", self.paths.downloads().display()));
                }
                for package in cached {
                    Self::line(output, format!("  {}", Self::add_package_line(&package)));
                }
                Self::line(output, "or reinstall Minecraft from the Microsoft Store. Then delete the journal file.");
                CliExit::RestoreFailed
            }
            other => {
                Self::line(output, format!("Error: {other}"));
                CliExit::Error
            }
        }
    }

    // the line that registers a package over any other version; the path is a PowerShell literal
    fn add_package_line(package: &Path) -> String {
        format!(
            "Add-AppxPackage -ForceUpdateFromAnyVersion -Path {}",
            PowerShellText::quote(&package.display().to_string())
        )
    }

    fn cached_packages(&self) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = match std::fs::read_dir(self.paths.downloads()) {
            Ok(read) => read
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .filter(|path| path.extension().is_some_and(|extension| extension == "msixvc"))
                .collect(),
            Err(_) => Vec::new(),
        };
        found.sort();
        found
    }

    fn list(&self, channel: Channel, output: &mut impl Write) -> Result<(), GdkError> {
        let registered = self.inventory.registered(channel.family_name())?.map(|package| package.version);
        let stored: Vec<GameVersion> = self.store.list()?.iter().map(|folder| folder.version).collect();
        for entry in self.catalogue.entries(channel) {
            let mut marks = Vec::new();
            if registered == Some(entry.version) {
                marks.push("installed in the Store");
            }
            if stored.contains(&entry.version) {
                marks.push("in the version store");
            }
            let marks = if marks.is_empty() { String::new() } else { format!("  ({})", marks.join(", ")) };
            Self::line(output, format!("{}{marks}", entry.version));
        }
        Ok(())
    }

    fn install(
        &self,
        version: &str,
        channel: Channel,
        input: &mut impl BufRead,
        output: &mut impl Write,
    ) -> Result<(), GdkError> {
        let request = InstallRequest { version: GameVersion::parse(version)?, channel };
        let mut offered = false;
        loop {
            match self.install_once(&request, output) {
                Ok(outcome) => {
                    Self::summarize(&outcome, output);
                    return Ok(());
                }
                Err(error @ GdkError::ExeCopyFailed { .. }) if !offered => {
                    offered = true;
                    if self.developer_mode.is_on().unwrap_or(true) || !self.ask_developer_mode(input, output) {
                        return Err(error);
                    }
                    if !self.developer_mode.enable_with_consent()? {
                        Self::line(output, "Developer Mode is still off.");
                        return Err(error);
                    }
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn install_once(&self, request: &InstallRequest, output: &mut impl Write) -> Result<InstallOutcome, GdkError> {
        let mut last_tenth: Option<(InstallStep, u64)> = None;
        self.installer.install(request, &mut |step, progress| match progress {
            None => Self::line(output, step.label()),
            Some(progress) => {
                let tenth = progress.bytes_done * 10 / progress.bytes_total.max(1);
                if last_tenth != Some((step, tenth)) {
                    last_tenth = Some((step, tenth));
                    Self::line(output, format!("  {} {}%", step.label(), tenth * 10));
                }
            }
        })
    }

    fn ask_developer_mode(&self, input: &mut impl BufRead, output: &mut impl Write) -> bool {
        Self::line(output, "Copying the game executable failed.");
        let _ = write!(output, "Turn on Developer Mode (UAC prompt) and retry? [y/N] ");
        let _ = output.flush();
        let mut answer = String::new();
        let _ = input.read_line(&mut answer);
        answer.trim().eq_ignore_ascii_case("y")
    }

    fn summarize(outcome: &InstallOutcome, output: &mut impl Write) {
        Self::line(
            output,
            format!(
                "Installed {} at {} ({} files, {})",
                outcome.version.version,
                outcome.version.path.display(),
                outcome.version.files,
                outcome.path_used
            ),
        );
        if outcome.store_updated_meanwhile {
            Self::line(
                output,
                format!("The Store updated Minecraft meanwhile; {} stays installed.", outcome.final_full_name),
            );
        }
    }

    fn instance(&self, version: &str, name: &str, output: &mut impl Write) -> Result<(), GdkError> {
        let folder = self.instances.create(&GameVersion::parse(version)?, name)?;
        Self::line(output, format!("Created {}", folder.path.display()));
        Ok(())
    }

    fn seed(&self, instance_dir: &Path, output: &mut impl Write) -> Result<(), GdkError> {
        let text = std::fs::read_to_string(instance_dir.join("version.json"))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|error| GdkError::BadVersion(error.to_string()))?;
        let raw = json.get("gameVersion").and_then(|value| value.as_str()).unwrap_or_default();
        let version = GameVersion::parse(raw)?;
        let folder = self.instances.seed_version_from_instance(instance_dir, &version)?;
        Self::line(output, format!("Stored {} at {} ({} files)", version, folder.path.display(), folder.files));
        Ok(())
    }

    fn recover(&self, output: &mut impl Write) -> Result<(), GdkError> {
        match self.installer.recover()? {
            Some(restored) => Self::line(output, format!("Restored {restored}")),
            None => Self::line(output, "Nothing to recover."),
        }
        Ok(())
    }

    fn status(&self, output: &mut impl Write) -> Result<(), GdkError> {
        for channel in [Channel::Release, Channel::Preview] {
            match self.inventory.registered(channel.family_name())? {
                Some(package) => Self::line(
                    output,
                    format!(
                        "{:?}: {} ({})",
                        channel,
                        package.full_name,
                        if package.status_ok { "Ok" } else { "not healthy" }
                    ),
                ),
                None => Self::line(output, format!("{channel:?}: not installed")),
            }
        }
        let developer_mode = if self.developer_mode.is_on()? { "on" } else { "off" };
        Self::line(output, format!("Developer Mode: {developer_mode}"));
        for folder in self.store.list()? {
            Self::line(output, format!("Stored: {} ({} files)", folder.version, folder.files));
        }
        let journal = if self.paths.journal().exists() { "present (run recover)" } else { "none" };
        Self::line(output, format!("Install journal: {journal}"));
        Ok(())
    }
}

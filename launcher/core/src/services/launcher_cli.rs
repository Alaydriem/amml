use crate::{
    Adopter, AmmlCommand, AmmlState, CliArgs, CliCommand, CliProgress, CoreProgress, CoreServices, DataCommand,
    DataMigrator, InstanceCommand, InstanceReader, InstanceState, LaunchService, LaunchState, LauncherError,
    LauncherPaths, LauncherSettings, ModChange, ModCommand, ModId, NewInstanceRequest, PreviewSwitch,
    ProcessGameStarter, RegistryCommand, RegistryRef, RegistryStatus, SettingsCommand, SettingsStore, UpdateChecker,
    UpdateRow, UpdateStatus, UrlInspector, WindowsWindowProbe,
};
use amml_gdk::{AmmlPaths, CancelFlag, DownloadProgress, GameVersion, InstanceService, VersionStore};
use std::io::Write;
use std::time::Duration;

// The `amml-launcher` command line. It writes to the stream it is given, so a test reads its output.
pub struct LauncherCli {
    paths: LauncherPaths,
    services: CoreServices,
}

impl LauncherCli {
    // exit code 2 for the refusals a caller is expected to handle, 1 for every other error
    const REFUSED: u8 = 2;

    // 0 on success; on an error one `error: <message>` line is written and the code is 1, or 2 for a refusal
    pub fn run(args: CliArgs, out: &mut impl Write) -> i32 {
        let outcome = Self::new(&args).and_then(|cli| cli.dispatch(&args.command, out));
        match outcome {
            Ok(code) => code,
            Err(error) => {
                let _ = writeln!(out, "error: {error}");
                Self::exit_code(&error)
            }
        }
    }

    fn new(args: &CliArgs) -> Result<LauncherCli, LauncherError> {
        let root = match &args.root {
            Some(root) => root.clone(),
            None => AmmlPaths::default_root()?,
        };
        #[cfg(debug_assertions)]
        let official =
            args.official.as_ref().map(|base| if base.ends_with('/') { base.clone() } else { format!("{base}/") });
        #[cfg(not(debug_assertions))]
        let official = None;
        let paths = LauncherPaths::new(AmmlPaths::new(root));
        Ok(LauncherCli { services: CoreServices::new(paths.clone(), official, CancelFlag::new()), paths })
    }

    fn exit_code(error: &LauncherError) -> i32 {
        match error {
            LauncherError::InstanceRunning(_) | LauncherError::NotSupported { .. } | LauncherError::CoreMod(_) => {
                i32::from(Self::REFUSED)
            }
            _ => 1,
        }
    }

    fn dispatch(&self, command: &CliCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        match command {
            CliCommand::Registry { command } => self.registry(command, out),
            CliCommand::Instance { command } => self.instance(command, out),
            CliCommand::Amml { command } => self.amml(command, out),
            CliCommand::Mod { command } => self.mods(command, out),
            CliCommand::Updates { instance } => self.updates(instance, out),
            CliCommand::Data { command } => self.data(command, out),
            CliCommand::Launch { instance, window_timeout } => self.launch(instance, *window_timeout, out),
            CliCommand::Settings { command } => self.settings(command, out),
        }
    }

    fn registry(&self, command: &RegistryCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        let mut catalogue = self.services.catalogue()?;
        match command {
            RegistryCommand::Status => {
                for client in catalogue.registries_mut().clients_mut() {
                    let status = client.refresh();
                    Self::line(
                        out,
                        format!("{} ({}): {}", client.name(), client.registry().base, Self::status_text(&status)),
                    );
                    if let Some(root) = client.root() {
                        Self::line(out, format!("  mods: {}", root.mod_count));
                        let versions: Vec<String> = root
                            .amml
                            .versions
                            .iter()
                            .map(|release| format!("{} (API {})", release.version, release.api_version))
                            .collect();
                        Self::line(out, format!("  AMML: {}", versions.join(", ")));
                    }
                }
            }
            RegistryCommand::Search { query } => {
                for (registry, hits) in catalogue.registries_mut().search(query) {
                    Self::line(out, registry.base);
                    for hit in hits {
                        Self::line(out, format!("  {}  {}  (API {})", hit.id, hit.title, hit.api));
                    }
                }
            }
            RegistryCommand::Show { id } => {
                let id = ModId::parse(id)?;
                let mut shown = false;
                for client in catalogue.registries_mut().clients_mut() {
                    let Ok(cached) = client.document(&id) else {
                        continue;
                    };
                    let document = cached.value;
                    Self::line(out, format!("{}  {}", document.id, document.listing.title));
                    Self::line(out, format!("  {}", document.listing.summary));
                    Self::line(out, format!("  registry: {}", client.registry().base));
                    Self::line(out, format!("  core: {}  default: {}", document.core, document.default));
                    for release in &document.versions {
                        Self::line(
                            out,
                            format!("  {}  API {}  {} bytes", release.version, release.api_version, release.size),
                        );
                    }
                    shown = true;
                    break;
                }
                if !shown {
                    return Err(LauncherError::UnknownMod(id.as_str()));
                }
            }
        }
        Ok(0)
    }

    fn instance(&self, command: &InstanceCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        match command {
            InstanceCommand::List => {
                for listed in InstanceReader::list(&self.paths) {
                    match listed {
                        Ok(state) => Self::line(
                            out,
                            format!(
                                "{}  {}  amml {}  mods {}",
                                state.name,
                                state.game.instance_name(),
                                AmmlState::label(state.amml.version.as_deref(), state.amml.local),
                                state.mods.len()
                            ),
                        ),
                        Err(error) => Self::line(out, format!("skipped: {error}")),
                    }
                }
            }
            InstanceCommand::Show { name } => {
                let state = InstanceReader::read(&self.paths.checked_instance(name)?)?;
                Self::show(&state, out);
            }
            InstanceCommand::New { game, name, preview, amml, own_data } => {
                let settings = self.services.settings()?;
                let request = NewInstanceRequest {
                    game: GameVersion::parse(game)?,
                    preview: *preview,
                    name: name.clone(),
                    amml: amml.clone(),
                    data: self.services.data_choice(*own_data, &settings),
                };
                let mut manager = self.services.manager()?;
                let mut printer = CliProgress::new();
                let outcome = manager.create(&request, &mut |event| Self::progress(&mut printer, &event, out))?;
                Self::line(out, format!("Created {} ({})", outcome.state.name, outcome.state.folder.display()));
                for (id, reason) in &outcome.skipped_mods {
                    Self::line(out, format!("skipped {id}: {reason}"));
                }
            }
            InstanceCommand::Delete { name, confirm } => {
                self.services.manager()?.delete(name, confirm)?;
                Self::line(out, format!("Deleted {name}"));
            }
            InstanceCommand::Adopt { name: None, .. } => {
                for candidate in Adopter::candidates(&self.paths) {
                    Self::line(
                        out,
                        format!(
                            "{}  {}  amml {}  mods {}  {}",
                            candidate.name,
                            candidate.game.instance_name(),
                            AmmlState::label(candidate.amml.as_deref(), candidate.amml_local),
                            candidate.mod_count,
                            if candidate.has_version_in_store { "version in store" } else { "version not in store" }
                        ),
                    );
                }
            }
            InstanceCommand::Adopt { name: Some(name), own_data } => {
                let settings = self.services.settings()?;
                let gdk = self.paths.gdk().clone();
                let instances = InstanceService::new(gdk.clone(), VersionStore::new(gdk.clone()));
                let state = Adopter::adopt(
                    &self.paths,
                    &instances,
                    &VersionStore::new(gdk),
                    name,
                    &self.services.data_choice(*own_data, &settings),
                )?;
                Self::line(out, format!("Adopted {}", state.name));
                let mut printer = CliProgress::new();
                let added = self
                    .services
                    .manager()?
                    .ensure_core_mods(name, &mut |event| Self::progress(&mut printer, &event, out))?;
                for id in added {
                    Self::line(out, format!("added core mod {id}"));
                }
            }
        }
        Ok(0)
    }

    fn amml(&self, command: &AmmlCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        let AmmlCommand::Update { instance, to } = command;
        let mut printer = CliProgress::new();
        let version = self
            .services
            .manager()?
            .update_amml(instance, to.clone(), &mut |event| Self::progress(&mut printer, &event, out))?;
        Self::line(out, format!("AMML {version} is installed in {instance}"));
        Ok(0)
    }

    fn mods(&self, command: &ModCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        let mut manager = self.services.manager()?;
        match command {
            ModCommand::Add { instance, target, unofficial_ok } => {
                let mut printer = CliProgress::new();
                if target.starts_with("http://") {
                    return Err(LauncherError::Http {
                        url: target.clone(),
                        text: "only https URLs are accepted".to_string(),
                    });
                }
                if target.starts_with("https://") {
                    let inspection = UrlInspector::inspect(manager.catalogue_mut(), target, &mut |download| {
                        Self::download(&mut printer, "Downloading", download, out);
                    })?;
                    let id = inspection.id.as_str();
                    let change = manager.install_inspected(instance, inspection, *unofficial_ok)?;
                    Self::line(out, Self::change_text(&id, &change));
                } else {
                    let id = ModId::parse(target)?;
                    let resolved = manager.resolve_mod(instance, &id, None)?;
                    let change = manager
                        .install_mod(instance, &resolved, &mut |event| Self::progress(&mut printer, &event, out))?;
                    Self::line(out, Self::change_text(&id.as_str(), &change));
                }
            }
            ModCommand::Update { instance, id, all } => {
                let rows = manager.check_updates(instance)?;
                let wanted: Vec<&UpdateRow> = rows
                    .iter()
                    .filter(|row| row.id != UpdateChecker::AMML_ID)
                    .filter(|row| *all || id.as_deref() == Some(row.id.as_str()))
                    .collect();
                if wanted.is_empty() {
                    return Err(LauncherError::UnknownMod(id.clone().unwrap_or_default()));
                }
                let mut printer = CliProgress::new();
                let held = InstanceReader::read(&self.paths.checked_instance(instance)?)?;
                for row in wanted {
                    if row.status != UpdateStatus::Available {
                        Self::line(out, format!("{} {}", row.id, Self::status_word(row.status)));
                        continue;
                    }
                    let mod_id = ModId::parse(&row.id)?;
                    let source = held
                        .mods
                        .iter()
                        .find(|installed| installed.id == mod_id)
                        .map(|installed| installed.source.clone());
                    let resolved = manager.resolve_mod(instance, &mod_id, source.as_ref())?;
                    let change = manager
                        .install_mod(instance, &resolved, &mut |event| Self::progress(&mut printer, &event, out))?;
                    Self::line(out, Self::change_text(&row.id, &change));
                }
            }
            ModCommand::Off { instance, id } => {
                manager.set_mod_enabled(instance, &ModId::parse(id)?, false)?;
                Self::line(out, format!("{id} is off in {instance}"));
            }
            ModCommand::On { instance, id } => {
                manager.set_mod_enabled(instance, &ModId::parse(id)?, true)?;
                Self::line(out, format!("{id} is on in {instance}"));
            }
            ModCommand::Remove { instance, id } => {
                manager.remove_mod(instance, &ModId::parse(id)?)?;
                Self::line(out, format!("{id} is removed from {instance}"));
            }
        }
        Ok(0)
    }

    fn updates(&self, instance: &str, out: &mut impl Write) -> Result<i32, LauncherError> {
        for row in self.services.manager()?.check_updates(instance)? {
            Self::line(
                out,
                format!(
                    "{}  {}  {}  {}{}",
                    row.id,
                    row.installed.as_deref().unwrap_or("-"),
                    row.available.as_deref().unwrap_or("-"),
                    Self::status_word(row.status),
                    if row.api_change { "  api-change" } else { "" }
                ),
            );
        }
        Ok(0)
    }

    fn data(&self, command: &DataCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        let migrator = DataMigrator::new(DataMigrator::appdata_bedrock().unwrap_or_default(), self.paths.clone());
        match command {
            DataCommand::Candidates => {
                for source in migrator.candidates() {
                    Self::line(
                        out,
                        format!("{}  {} worlds  {}", source.label, source.world_count, source.path.display()),
                    );
                }
            }
            DataCommand::Copy { source } => {
                let data_dir = self.services.settings()?.data_dir(&self.paths);
                let mode = migrator.check(source, &data_dir)?;
                let plan = migrator.measure(source, &data_dir, mode)?;
                migrator.copy(&plan, &mut |_, _| {})?;
                Self::line(
                    out,
                    format!("Copied {} files to {}", plan.files.len(), data_dir.join("Minecraft Bedrock").display()),
                );
            }
        }
        Ok(0)
    }

    fn launch(&self, instance: &str, window_timeout: Option<u64>, out: &mut impl Write) -> Result<i32, LauncherError> {
        let state = InstanceReader::read(&self.paths.checked_instance(instance)?)?;
        let timeout = window_timeout
            .map(Duration::from_secs)
            .unwrap_or(LaunchService::<ProcessGameStarter, WindowsWindowProbe>::DEFAULT_WINDOW_TIMEOUT);
        let mut service = LaunchService::new(ProcessGameStarter::new(), WindowsWindowProbe::new(), timeout);
        match service.launch(&state)? {
            LaunchState::Window { pid, seconds } => {
                Self::line(out, format!("{instance} shows a window (pid {pid}, after {seconds} s)"));
                Ok(0)
            }
            LaunchState::NoWindow { pid } => {
                Self::line(out, format!("{instance} runs (pid {pid}) but showed no window in time"));
                Ok(0)
            }
            LaunchState::Exited { code, seconds } => {
                let code = code.map(|code| code.to_string()).unwrap_or_else(|| "unknown".to_string());
                Self::line(out, format!("{instance} ended after {seconds} s with exit code {code}"));
                Ok(1)
            }
        }
    }

    fn settings(&self, command: &SettingsCommand, out: &mut impl Write) -> Result<i32, LauncherError> {
        let store = SettingsStore::new(self.paths.clone());
        let mut settings = store.load()?;
        match command {
            SettingsCommand::Show => {
                for registry in &settings.registries {
                    Self::line(out, format!("registry {}", registry.base));
                }
                Self::line(out, format!("preview {}", if settings.preview { "on" } else { "off" }));
                Self::line(out, format!("data_dir {}", settings.data_dir(&self.paths).display()));
            }
            SettingsCommand::AddRegistry { url } => {
                settings.registries.push(RegistryRef::parse(url)?);
                store.save(&settings)?;
                Self::line(out, format!("Added {url}"));
            }
            SettingsCommand::RemoveRegistry { url } => {
                let target = RegistryRef::parse(url)?;
                if target.base == LauncherSettings::OFFICIAL_REGISTRY {
                    return Err(LauncherError::BadSettings("the official registry cannot be removed".to_string()));
                }
                settings.registries.retain(|registry| registry.base != target.base);
                store.save(&settings)?;
                Self::line(out, format!("Removed {url}"));
            }
            SettingsCommand::Preview { switch } => {
                settings.preview = *switch == PreviewSwitch::On;
                store.save(&settings)?;
                Self::line(out, format!("preview {}", if settings.preview { "on" } else { "off" }));
            }
        }
        Ok(0)
    }

    fn show(state: &InstanceState, out: &mut impl Write) {
        Self::line(
            out,
            format!("{}  {}{}", state.name, state.game.instance_name(), if state.preview { " (preview)" } else { "" }),
        );
        Self::line(out, format!("  folder: {}", state.folder.display()));
        Self::line(out, format!("  amml: {}", AmmlState::label(state.amml.version.as_deref(), state.amml.local)));
        Self::line(out, format!("  managed: {}", state.managed));
        if let Some(data_dir) = &state.data_dir {
            Self::line(out, format!("  data: {}", data_dir.display()));
        }
        for installed in &state.mods {
            Self::line(
                out,
                format!(
                    "  {}  {}  {}  {}",
                    installed.id.as_str(),
                    installed.version.as_deref().unwrap_or("-"),
                    if installed.enabled { "on" } else { "off" },
                    installed.source.as_text()
                ),
            );
        }
    }

    fn status_text(status: &RegistryStatus) -> String {
        match status {
            RegistryStatus::Ok => "ok".to_string(),
            RegistryStatus::Stale { reason } => format!("stale ({reason})"),
            RegistryStatus::Offline => "offline".to_string(),
            RegistryStatus::Broken { reason } => format!("broken ({reason})"),
        }
    }

    fn status_word(status: UpdateStatus) -> &'static str {
        match status {
            UpdateStatus::UpToDate => "up-to-date",
            UpdateStatus::Available => "available",
            UpdateStatus::Unknown => "unknown",
            UpdateStatus::NotListed => "not-listed",
            UpdateStatus::Unofficial => "unofficial",
            UpdateStatus::Offline => "offline",
            UpdateStatus::LocalBuild => "local-build",
            UpdateStatus::NeedsAmml => "needs-amml",
        }
    }

    fn change_text(id: &str, change: &ModChange) -> String {
        match change {
            ModChange::Installed => format!("Installed {id}"),
            ModChange::Updated { from: Some(from) } => format!("Updated {id} (was {from})"),
            ModChange::Updated { from: None } => format!("Updated {id}"),
        }
    }

    fn line(out: &mut impl Write, text: impl std::fmt::Display) {
        let _ = writeln!(out, "{text}");
    }

    fn progress(printer: &mut CliProgress, event: &CoreProgress, out: &mut impl Write) {
        if let Some(text) = printer.line(event) {
            Self::line(out, text);
        }
    }

    fn download(printer: &mut CliProgress, label: &str, progress: DownloadProgress, out: &mut impl Write) {
        if let Some(text) = printer.download(label, progress) {
            Self::line(out, text);
        }
    }
}

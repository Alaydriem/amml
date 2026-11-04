use amml_gdk::{
    AmmlPaths, CancelFlag, Catalogue, CatalogueReader, CliApp, CliArgs, CliCommand, CliExit, DeveloperMode, DiskSpace,
    DownloadCache, Downloader, GdkError, InstallService, StoreCheck, WindowsExeCopier, WindowsPackageDeployer,
    WindowsPackageInventory, WindowsProcessInspector,
};
use clap::Parser;
use std::io::Write;
use std::time::Duration;

struct Launcher;

impl Launcher {
    fn agent() -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(30)))
            .timeout_recv_response(Some(Duration::from_secs(60)))
            .timeout_per_read(Some(Duration::from_secs(60)))
            .build()
            .into()
    }

    fn run(command: &CliCommand) -> Result<CliExit, GdkError> {
        let paths = AmmlPaths::new(AmmlPaths::default_root()?);
        let agent = Self::agent();
        let catalogue = if command.needs_catalogue() {
            CatalogueReader::fetch(&agent)?
        } else {
            Catalogue::new(Vec::new(), Vec::new())
        };

        let installer = InstallService::new(
            paths.clone(),
            catalogue.clone(),
            Downloader::new(agent, paths.downloads(), Downloader::DEFAULT_ATTEMPTS, CancelFlag::new()),
            DownloadCache::new(paths.downloads()),
            StoreCheck::new(WindowsPackageInventory::new(), WindowsProcessInspector::new(), DiskSpace::new()),
            WindowsPackageInventory::new(),
            WindowsPackageDeployer::new(),
            WindowsExeCopier::new(WindowsExeCopier::DEFAULT_TIMEOUT),
            Duration::from_secs(10),
        );
        let app = CliApp::new(installer, WindowsPackageInventory::new(), DeveloperMode::new(), paths, catalogue);
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let exit = app.run(command, &mut stdin.lock(), &mut stdout);
        let _ = stdout.flush();
        Ok(exit)
    }
}

fn main() {
    let args = CliArgs::parse();
    let code = match Launcher::run(&args.command) {
        Ok(exit) => exit.code(),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "Error: {error}");
            CliExit::Error.code()
        }
    };
    std::process::exit(code);
}

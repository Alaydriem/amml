use amml_gdk::GameVersion;
use amml_registry::{AmmlRelease, ModDocument, ModRelease, RegistryRoot, ReleaseVersion};

// Which AMML release fits a game version, and which mod release fits an API.
pub struct Compatibility;

impl Compatibility {
    // the newest AMML release that lists the game; an entry that is not a game version is skipped
    pub fn amml_for(game: &GameVersion, root: &RegistryRoot) -> Option<AmmlRelease> {
        root.amml
            .versions
            .iter()
            .filter(|release| {
                release
                    .game_versions
                    .iter()
                    .filter_map(|entry| GameVersion::parse(entry).ok())
                    .any(|listed| listed == *game)
            })
            .filter_map(|release| ReleaseVersion::parse(&release.version).ok().map(|version| (version, release)))
            .max_by_key(|(version, _)| *version)
            .map(|(_, release)| release.clone())
    }

    pub fn newest_for_api(document: &ModDocument, api: u32) -> Option<ModRelease> {
        document
            .versions
            .iter()
            .filter(|release| release.api_version == api)
            .filter_map(|release| ReleaseVersion::parse(&release.version).ok().map(|version| (version, release)))
            .max_by_key(|(version, _)| *version)
            .map(|(_, release)| release.clone())
    }

    // every game version some AMML release lists, newest first
    pub fn installable_games(root: &RegistryRoot) -> Vec<GameVersion> {
        let mut games: Vec<GameVersion> = root
            .amml
            .versions
            .iter()
            .flat_map(|release| release.game_versions.iter())
            .filter_map(|entry| GameVersion::parse(entry).ok())
            .collect();
        games.sort_by(|a, b| b.cmp(a));
        games.dedup();
        games
    }
}

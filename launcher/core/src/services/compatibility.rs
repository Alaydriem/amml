use crate::structs::{AmmlLevel, NeedsAmml, ReleaseChoice};
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

    // a release with no minimum fits all; a local build meets every minimum;
    // an unknown level, or a minimum that does not parse, fits nothing
    pub fn fits(release: &ModRelease, level: &AmmlLevel) -> bool {
        Self::fits_min(release.min_amml.as_deref(), level)
    }

    // the same rule for a bare minimum, as an index card holds it
    pub fn fits_min(min: Option<&str>, level: &AmmlLevel) -> bool {
        let Some(min) = min else {
            return true;
        };
        match level {
            AmmlLevel::Local => true,
            AmmlLevel::Unknown => false,
            AmmlLevel::Release(have) => ReleaseVersion::parse(min).is_ok_and(|need| need <= *have),
        }
    }

    // the newest release for the API that fits the level; `blocked` names the minimum of a newer release that does not
    pub fn choose(document: &ModDocument, api: u32, level: &AmmlLevel, root: Option<&RegistryRoot>) -> ReleaseChoice {
        let mut releases: Vec<(ReleaseVersion, &ModRelease)> = document
            .versions
            .iter()
            .filter(|release| release.api_version == api)
            .filter_map(|release| ReleaseVersion::parse(&release.version).ok().map(|version| (version, release)))
            .collect();
        releases.sort_by_key(|entry| std::cmp::Reverse(entry.0));
        let fitting = releases.iter().find(|(_, release)| Self::fits(release, level)).map(|(_, release)| *release);
        let blocked = match releases.first() {
            Some((_, newest)) if !Self::fits(newest, level) => {
                newest.min_amml.as_deref().map(|min| Self::needs(min, root))
            }
            _ => None,
        };
        ReleaseChoice { fitting: fitting.cloned(), blocked }
    }

    // the newest game of the AMML release whose version equals `min`, by number; `None` when the root lacks it
    pub fn needs(min: &str, root: Option<&RegistryRoot>) -> NeedsAmml {
        let game = ReleaseVersion::parse(min).ok().and_then(|wanted| {
            root?
                .amml
                .versions
                .iter()
                .find(|release| ReleaseVersion::parse(&release.version).is_ok_and(|version| version == wanted))?
                .game_versions
                .iter()
                .filter_map(|entry| GameVersion::parse(entry).ok().map(|parsed| (parsed, entry)))
                .max_by(|a, b| a.0.cmp(&b.0))
                .map(|(_, entry)| entry.clone())
        });
        NeedsAmml { version: min.to_string(), game }
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

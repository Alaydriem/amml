use crate::{ChannelView, VersionStateView, VersionView};
use amml_gdk::{Catalogue, Channel, GameVersion, VersionFolder};
use amml_launcher_core::InstanceState;
use amml_registry::{AmmlRelease, ReleaseVersion};

// The game versions the player can pick: the catalogue's, plus any in the store that it does not list.
pub struct VersionList;

impl VersionList {
    // Newest first. A version no AMML release lists is `unsupported`, whether or not it is in the store. A listed version
    // is `installed` when it is in the store, else `download`.
    pub fn build(
        catalogue: &Catalogue,
        store: &[VersionFolder],
        amml: &[AmmlRelease],
        instances: &[InstanceState],
        defaults: u32,
    ) -> Vec<VersionView> {
        let mut found: Vec<(GameVersion, VersionView)> = Vec::new();
        for channel in [Channel::Release, Channel::Preview] {
            for entry in catalogue.entries(channel) {
                found.push((entry.version, Self::view(entry.version, channel, store, amml, instances, defaults)));
            }
        }
        for folder in store {
            if found.iter().any(|(version, _)| *version == folder.version) {
                continue;
            }
            let preview = instances.iter().any(|state| state.game == folder.version && state.preview);
            let channel = if preview { Channel::Preview } else { Channel::Release };
            found.push((folder.version, Self::view(folder.version, channel, store, amml, instances, defaults)));
        }
        found.sort_by_key(|left| std::cmp::Reverse(left.0));
        found.into_iter().map(|(_, view)| view).collect()
    }

    // Every version in the store with its size, newest first, whether or not an AMML release lists it. This is the list
    // of the game versions settings page.
    pub fn on_disk(store: &[VersionFolder], instances: &[InstanceState]) -> Vec<VersionView> {
        let mut folders: Vec<&VersionFolder> = store.iter().collect();
        folders.sort_by_key(|folder| std::cmp::Reverse(folder.version));
        folders
            .into_iter()
            .map(|folder| {
                let preview = instances.iter().any(|state| state.game == folder.version && state.preview);
                VersionView {
                    version: folder.version.instance_name(),
                    channel: if preview { ChannelView::Preview } else { ChannelView::Release },
                    state: VersionStateView::Installed,
                    size: Some(folder.bytes),
                    instance_count: instances.iter().filter(|instance| instance.game == folder.version).count() as u32,
                    amml: None,
                    default_mods: 0,
                }
            })
            .collect()
    }

    fn view(
        version: GameVersion,
        channel: Channel,
        store: &[VersionFolder],
        amml: &[AmmlRelease],
        instances: &[InstanceState],
        defaults: u32,
    ) -> VersionView {
        let newest = Self::newest_amml(&version, amml);
        let folder = store.iter().find(|folder| folder.version == version);
        // a version no AMML release lists cannot make an instance, even when it is in the store
        let state = match (&newest, folder) {
            (None, _) => VersionStateView::Unsupported,
            (Some(_), Some(_)) => VersionStateView::Installed,
            (Some(_), None) => VersionStateView::Download,
        };
        VersionView {
            version: version.instance_name(),
            channel: match channel {
                Channel::Release => ChannelView::Release,
                Channel::Preview => ChannelView::Preview,
            },
            state,
            size: folder.map(|folder| folder.bytes),
            instance_count: instances.iter().filter(|instance| instance.game == version).count() as u32,
            default_mods: if newest.is_some() { defaults } else { 0 },
            amml: newest.map(|release| release.version),
        }
    }

    // the newest AMML release that lists the game
    fn newest_amml(version: &GameVersion, amml: &[AmmlRelease]) -> Option<AmmlRelease> {
        amml.iter()
            .filter(|release| {
                release
                    .game_versions
                    .iter()
                    .filter_map(|entry| GameVersion::parse(entry).ok())
                    .any(|listed| listed == *version)
            })
            .filter_map(|release| ReleaseVersion::parse(&release.version).ok().map(|parsed| (parsed, release)))
            .max_by_key(|(parsed, _)| *parsed)
            .map(|(_, release)| release.clone())
    }
}

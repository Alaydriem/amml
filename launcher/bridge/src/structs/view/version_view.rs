use super::{ChannelView, VersionStateView};
use serde::Serialize;
use ts_rs::TS;

// A game version the player can pick or has on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VersionView {
    pub version: String,
    pub channel: ChannelView,
    pub state: VersionStateView,
    #[ts(type = "number | null")]
    pub size: Option<u64>,
    pub instance_count: u32,
    // the newest AMML release for this version
    pub amml: Option<String>,
    pub default_mods: u32,
}

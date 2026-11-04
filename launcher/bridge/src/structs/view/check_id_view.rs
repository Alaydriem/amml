use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CheckIdView {
    Store,
    GamingServices,
    Gameinput,
    Preview,
    Disk,
}

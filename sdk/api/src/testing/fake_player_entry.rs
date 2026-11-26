use crate::Uuid;
use crate::testing::FakeSkin;

/// One entry of a `FakePlayerList`. A Remove list gives only the uuid of each entry.
#[derive(Clone, Debug)]
pub struct FakePlayerEntry {
    pub uuid: Uuid,
    pub actor_id: i64,
    pub name: String,
    pub xuid: String,
    pub skin: Option<FakeSkin>,
}

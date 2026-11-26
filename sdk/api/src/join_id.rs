/// The id of one server join, from `Client::join_server`. `JoinServerResult.join` carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JoinId(pub u64);

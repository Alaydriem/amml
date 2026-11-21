/// The server of `Client::join_server`: `host` (a name or an address) and `port`. `name` is the server name the
/// game shows; `None` shows the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JoinTarget<'a> {
    pub host: &'a str,
    pub port: u16,
    pub name: Option<&'a str>,
}

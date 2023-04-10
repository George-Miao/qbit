#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct TransferInfo {
    /// Global download rate (bytes/s)
    pub dl_info_speed: u64,
    /// Data downloaded this session (bytes)
    pub dl_info_data: u64,
    /// Global upload rate (bytes/s)
    pub up_info_speed: u64,
    /// Data uploaded this session (bytes)
    pub up_info_data: u64,
    /// Download rate limit (bytes/s)
    pub dl_rate_limit: u64,
    /// Upload rate limit (bytes/s)
    pub up_rate_limit: u64,
    /// DHT nodes connected to
    pub dht_nodes: u64,
    /// Connection status. Possible values: connected, disconnected, firewalled
    pub connection_status: ConnectionStatus,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Firewalled,
    #[serde(other)]
    Unknown,
}

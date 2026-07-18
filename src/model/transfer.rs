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

/// Global and alternative speed limits returned by `transfer/getSpeedLimits`.
/// All values are in KiB/s, `-1` means unlimited.
/// Added in qBittorrent 5.2.0 (Web API v2.16.0).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpeedLimits {
    /// Global upload limit (KiB/s, -1 = unlimited)
    #[serde(rename = "up_limit")]
    pub upload_limit: i64,
    /// Global download limit (KiB/s, -1 = unlimited)
    #[serde(rename = "dl_limit")]
    pub download_limit: i64,
    /// Alternative upload limit (KiB/s, -1 = unlimited)
    #[serde(rename = "alt_up_limit")]
    pub alternative_upload_limit: i64,
    /// Alternative download limit (KiB/s, -1 = unlimited)
    #[serde(rename = "alt_dl_limit")]
    pub alternative_download_limit: i64,
}

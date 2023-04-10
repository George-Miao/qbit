use std::net::IpAddr;

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]

pub struct Log {
    /// ID of the message
    pub id: u64,
    /// Text of the message
    pub message: String,
    /// Milliseconds since epoch
    pub timestamp: u64,
    /// Type of the message: Log::NORMAL: `1`, Log::INFO: `2`, Log::WARNING:
    /// `4`, Log::CRITICAL: `8`
    #[serde(rename = "type")]
    pub log_type: i8,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq, Eq)]
pub struct PeerLog {
    /// ID of the peer
    pub id: i64,
    /// IP of the peer
    pub ip: IpAddr,
    /// Milliseconds since epoch
    pub timestamp: u64,
    /// Whether or not the peer was blocked
    pub blocked: bool,
    /// Reason of the block
    pub reason: Option<String>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i8)]
pub enum LogLevel {
    Normal   = 1,
    Info     = 2,
    Warning  = 4,
    Critical = 8,
}

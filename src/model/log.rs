use std::net::IpAddr;

use serde_with::skip_serializing_none;

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

#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
#[skip_serializing_none]
pub struct GetLogsArg {
    /// Include normal messages (default: `true`)
    pub normal: Option<bool>,
    /// Include info messages (default: `true`)
    pub info: Option<bool>,
    /// Include warning messages (default: `true`)
    pub warning: Option<bool>,
    /// Include critical messages (default: `true`)
    pub critical: Option<bool>,
    /// Exclude messages with "message id" <= `last_known_id` (default: `-1`)
    pub last_known_id: Option<i64>,
}

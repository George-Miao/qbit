use std::{collections::HashMap, net::SocketAddr};

use serde_value::Value;

use crate::model::{Category, Torrent};

/// Main-data synchronization response.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct SyncData {
    /// Response ID
    pub rid: i64,
    /// Whether the response contains all the data or partial data
    pub full_update: Option<bool>,
    /// Property: torrent hash, value: same as [torrent list](#get-torrent-list)
    pub torrents: Option<HashMap<String, Torrent>>,
    /// List of hashes of torrents removed since last request
    pub torrents_removed: Option<Vec<String>>,
    /// Info for categories added since last request
    pub categories: Option<HashMap<String, Category>>,
    /// List of categories removed since last request
    pub categories_removed: Option<Vec<String>>,
    /// List of tags added since last request
    pub tags: Option<Vec<String>>,
    /// List of tags removed since last request
    pub tags_removed: Option<Vec<String>>,
    /// Map of trackers added since last request, and the torrents that have
    /// them. Property: tracker URL, value: torrent hash
    pub trackers: Option<HashMap<String, Vec<String>>>,
    /// List of tracker URLs removed since last request
    pub trackers_removed: Option<Vec<String>>,
    /// Global transfer info
    pub server_state: Option<HashMap<String, Value>>,
}

/// Peer synchronization response for a torrent.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct PeerSyncData {
    /// Whether the response contains a complete peer list.
    pub full_update: Option<bool>,
    /// Current peers keyed by socket address.
    pub peers: Option<HashMap<SocketAddr, Peer>>,
    /// Peer addresses removed since the previous response.
    pub peers_removed: Option<Vec<SocketAddr>>,
    /// Response ID to pass to the next synchronization request.
    pub rid: i64,
    /// Whether peer country flags should be displayed.
    pub show_flags: bool,
}
/// Peer transfer information returned by torrent peer synchronization.
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Peer {
    /// Peer client name.
    pub client: Option<String>,
    /// Connection type, such as TCP or uTP.
    pub connection: Option<String>,
    /// Peer country name.
    pub country: Option<String>,
    /// Peer country code.
    pub country_code: Option<String>,
    /// Peer download speed in bytes per second.
    pub dl_speed: Option<u64>,
    /// Bytes downloaded from the peer.
    pub downloaded: Option<u64>,
    /// Files requested by the peer.
    pub files: Option<String>,
    /// Compact peer flags.
    pub flags: Option<String>,
    /// Human-readable peer flag descriptions.
    pub flags_desc: Option<String>,
    /// Peer IP address.
    pub ip: Option<String>,
    /// Peer port.
    pub port: Option<u16>,
    /// Peer download progress as a fraction from zero to one.
    pub progress: Option<f64>,
    /// Relevance of this peer to the selected torrent files.
    pub relevance: Option<f64>,
    /// Peer upload speed in bytes per second.
    pub up_speed: Option<u64>,
    /// Bytes uploaded to the peer.
    pub uploaded: Option<u64>,
}

use std::{collections::HashMap, net::SocketAddr};

use serde_value::Value;

use crate::model::{Category, Torrent};

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

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct PeerSyncData {
    pub full_update: Option<bool>,
    pub peers: Option<HashMap<SocketAddr, Peer>>,
    pub peers_removed: Option<Vec<SocketAddr>>,
    pub rid: i64,
    pub show_flags: bool,
}
#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Peer {
    pub client: Option<String>,
    pub connection: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub dl_speed: Option<u64>,
    pub downloaded: Option<u64>,
    pub files: Option<String>,
    pub flags: Option<String>,
    pub flags_desc: Option<String>,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub progress: Option<f64>,
    pub relevance: Option<u64>,
    pub up_speed: Option<u64>,
    pub uploaded: Option<u64>,
}

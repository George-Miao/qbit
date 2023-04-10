use std::collections::HashMap;

use serde_value::Value;

use crate::model::{Category, Torrent};

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
pub struct Maindata {
    /// Response ID
    pub rid: i64,
    /// Whether the response contains all the data or partial data
    pub full_update: bool,
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
    /// Global transfer info
    pub server_state: Option<HashMap<String, Value>>,
}

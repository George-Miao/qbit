use serde::{Deserialize, Serialize};

/// Identifier returned when a search job is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SearchJob {
    pub id: i64,
}

/// Current state of a search job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchStatus {
    Running,
    Stopped,
}

/// Status and result count for a search job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchJobStatus {
    pub id: i64,
    pub status: SearchStatus,
    pub total: i64,
}

/// Results and state returned for a search job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub status: SearchStatus,
    pub total: i64,
}

/// A result produced by a qBittorrent search plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchResult {
    pub descr_link: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_url: String,
    pub nb_leechers: i64,
    pub nb_seeders: i64,
    pub site_url: String,
    pub engine_name: String,
    pub pub_date: i64,
}

/// A category supported by a search plugin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchPluginCategory {
    pub id: String,
    pub name: String,
}

/// Information about an installed search plugin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPlugin {
    pub enabled: bool,
    pub full_name: String,
    pub name: String,
    pub supported_categories: Vec<SearchPluginCategory>,
    pub url: String,
    pub version: String,
}

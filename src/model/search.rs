#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Identifier returned when a search job is started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SearchJob {
    /// Identifier of the search job.
    pub id: i64,
}

/// Current state of a search job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchStatus {
    /// The search is still running and its result count may increase.
    Running,
    /// The search has finished or was stopped.
    Stopped,
}

/// Status and result count for a search job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchJobStatus {
    /// Identifier of the search job.
    pub id: i64,
    /// Current state of the search job.
    pub status: SearchStatus,
    /// Total results found. This may increase while the job is running.
    pub total: i64,
}

/// Results and state returned for a search job.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchResults {
    /// Search results in the requested page.
    pub results: Vec<SearchResult>,
    /// Current state of the search job.
    pub status: SearchStatus,
    /// Total results found, before applying `limit` and `offset`.
    pub total: i64,
}

/// A result produced by a qBittorrent search plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SearchResult {
    /// URL of the torrent's description page.
    pub descr_link: String,
    /// Name of the result.
    pub file_name: String,
    /// Result size in bytes, or a negative value when unknown.
    pub file_size: i64,
    /// Torrent download URL or magnet URI.
    pub file_url: String,
    /// Number of leechers reported by the search plugin.
    pub nb_leechers: i64,
    /// Number of seeders reported by the search plugin.
    pub nb_seeders: i64,
    /// URL of the torrent site.
    pub site_url: String,
    /// Name of the search plugin that produced the result.
    pub engine_name: String,
    /// Publication time as seconds since the Unix epoch.
    pub pub_date: i64,
}

/// A category supported by a search plugin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchPluginCategory {
    /// Category identifier passed to the Search API.
    pub id: String,
    /// Human-readable category name.
    pub name: String,
}

/// Information about an installed search plugin.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPlugin {
    /// Whether the plugin is enabled.
    pub enabled: bool,
    /// Human-readable plugin name.
    pub full_name: String,
    /// Short plugin name used by Search API requests.
    pub name: String,
    /// Categories supported by the plugin.
    pub supported_categories: Vec<SearchPluginCategory>,
    /// URL of the torrent site searched by the plugin.
    pub url: String,
    /// Installed plugin version.
    pub version: String,
}

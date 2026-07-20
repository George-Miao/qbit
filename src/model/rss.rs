#![deny(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// An item in the recursive RSS hierarchy.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RssItem {
    /// A subscribed RSS feed.
    Feed(RssFeed),
    /// A folder containing feeds or nested folders, keyed by item name.
    Folder(HashMap<String, RssItem>),
}

/// An RSS feed and, when requested, its current articles.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssFeed {
    /// qBittorrent's unique identifier for the feed.
    pub uid: String,
    /// Source URL of the feed.
    pub url: String,
    /// Per-feed refresh interval in seconds, when one is configured.
    pub refresh_interval: Option<i64>,
    /// Feed title. Returned when `withData` is enabled.
    pub title: Option<String>,
    /// Feed-provided last build date. Returned when `withData` is enabled.
    pub last_build_date: Option<String>,
    /// Whether qBittorrent is currently loading the feed. Returned when
    /// `withData` is enabled.
    pub is_loading: Option<bool>,
    /// Whether the last feed load or parse failed. Returned when `withData` is
    /// enabled.
    pub has_error: Option<bool>,
    /// Current feed articles. Returned when `withData` is enabled.
    pub articles: Option<Vec<RssArticle>>,
}

/// An article returned as part of an RSS feed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RssArticle {
    /// Article identifier used by the RSS `markAsRead` endpoint.
    pub id: String,
    /// Article publication date in RFC 2822 form.
    pub date: String,
    /// Article title.
    pub title: String,
    /// Article author.
    pub author: String,
    /// Article description.
    pub description: String,
    /// Torrent URL extracted from the article, when available.
    #[serde(rename = "torrentURL")]
    pub torrent_url: String,
    /// Article link.
    pub link: String,
    /// Whether the article has been marked as read.
    pub is_read: bool,
}

/// Definition of an RSS auto-downloading rule.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RssRuleDefinition {
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Rule evaluation priority. Lower values are evaluated first.
    pub priority: i64,
    /// Text or expressions that a torrent name must contain.
    pub must_contain: String,
    /// Text or expressions that a torrent name must not contain.
    pub must_not_contain: String,
    /// Whether `mustContain` and `mustNotContain` use regular expressions.
    pub use_regex: bool,
    /// Smart episode-filter expression.
    pub episode_filter: String,
    /// Whether smart episode filtering is enabled.
    pub smart_filter: bool,
    /// Episode identifiers already matched by the smart filter.
    pub previously_matched_episodes: Vec<String>,
    /// Feed URLs to which the rule applies.
    pub affected_feeds: Vec<String>,
    /// Number of days for which subsequent matches are ignored.
    pub ignore_days: i64,
    /// Time of the rule's last match, formatted as an RFC 2822 date.
    pub last_match: String,
    /// Whether matched torrents are added in the stopped state. This is the
    /// legacy `addPaused` rule field.
    pub add_paused: Option<bool>,
    /// Category assigned to matched torrents.
    pub assigned_category: String,
    /// Save path assigned to matched torrents.
    pub save_path: String,
    /// Legacy content-layout setting for matched torrents.
    pub torrent_content_layout: Option<String>,
    /// Current qBittorrent add-torrent parameters for matched torrents.
    pub torrent_params: Option<serde_json::Value>,
}

impl Default for RssRuleDefinition {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 0,
            must_contain: String::new(),
            must_not_contain: String::new(),
            use_regex: false,
            episode_filter: String::new(),
            smart_filter: false,
            previously_matched_episodes: Vec::new(),
            affected_feeds: Vec::new(),
            ignore_days: 0,
            last_match: String::new(),
            add_paused: None,
            assigned_category: String::new(),
            save_path: String::new(),
            torrent_content_layout: None,
            torrent_params: None,
        }
    }
}

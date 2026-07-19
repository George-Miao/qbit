use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// An item in the recursive RSS hierarchy.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RssItem {
    Feed(RssFeed),
    Folder(HashMap<String, RssItem>),
}

/// An RSS feed and, when requested, its current articles.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssFeed {
    pub uid: String,
    pub url: String,
    pub refresh_interval: Option<i64>,
    pub title: Option<String>,
    pub last_build_date: Option<String>,
    pub is_loading: Option<bool>,
    pub has_error: Option<bool>,
    pub articles: Option<Vec<RssArticle>>,
}

/// An article returned as part of an RSS feed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RssArticle {
    pub id: String,
    pub date: String,
    pub title: String,
    pub author: String,
    pub description: String,
    #[serde(rename = "torrentURL")]
    pub torrent_url: String,
    pub link: String,
    pub is_read: bool,
}

/// Definition of an RSS auto-downloading rule.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RssRuleDefinition {
    pub enabled: bool,
    pub priority: i64,
    pub must_contain: String,
    pub must_not_contain: String,
    pub use_regex: bool,
    pub episode_filter: String,
    pub smart_filter: bool,
    pub previously_matched_episodes: Vec<String>,
    pub affected_feeds: Vec<String>,
    pub ignore_days: i64,
    pub last_match: String,
    pub add_paused: Option<bool>,
    pub assigned_category: String,
    pub save_path: String,
    pub torrent_content_layout: Option<String>,
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

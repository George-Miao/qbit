#![deny(missing_docs)]

use std::{borrow::Borrow, collections::HashMap};

use serde::Serialize;

use crate::{Qbit, Result, ext::*, model::*};

impl Qbit {
    /// Add a folder to the RSS hierarchy.
    pub async fn add_folder<T: AsRef<str> + Send + Sync>(&self, path: T) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            path: &'a str,
        }

        self.post_with(
            "rss/addFolder",
            &Arg {
                path: path.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Add an RSS feed, optionally under the supplied folder.
    pub async fn add_feed<T: AsRef<str> + Send + Sync>(
        &self,
        url: T,
        path: Option<T>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            url: &'a str,
            path: Option<&'a str>,
        }

        self.post_with(
            "rss/addFeed",
            &Arg {
                url: url.as_ref(),
                path: path.as_ref().map(AsRef::as_ref),
            },
        )
        .await?
        .end()
    }

    /// Remove an RSS feed or folder.
    pub async fn remove_item<T: AsRef<str> + Send + Sync>(&self, path: T) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            path: &'a str,
        }

        self.post_with(
            "rss/removeItem",
            &Arg {
                path: path.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Move an RSS feed or folder to another path.
    pub async fn move_item<T: AsRef<str> + Send + Sync>(
        &self,
        item_path: T,
        dest_path: T,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            item_path: &'a str,
            dest_path: &'a str,
        }

        self.post_with(
            "rss/moveItem",
            &Arg {
                item_path: item_path.as_ref(),
                dest_path: dest_path.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Return all RSS items as a recursive hierarchy.
    ///
    /// Set `with_data` to `true` to include each feed's current articles and
    /// loading state. With it disabled, qBittorrent returns feed identity and
    /// URL data without articles.
    ///
    /// See qBittorrent's [Get all items](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-all-items)
    /// documentation.
    pub async fn get_rss_items(&self, with_data: bool) -> Result<HashMap<String, RssItem>> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg {
            with_data: bool,
        }

        self.get_with("rss/items", &Arg { with_data })
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Mark an RSS item or article as read.
    pub async fn mark_as_read<T: AsRef<str> + Send + Sync>(
        &self,
        item_path: T,
        article_id: Option<T>,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            item_path: &'a str,
            article_id: Option<&'a str>,
        }

        self.post_with(
            "rss/markAsRead",
            &Arg {
                item_path: item_path.as_ref(),
                article_id: article_id.as_ref().map(AsRef::as_ref),
            },
        )
        .await?
        .end()
    }

    /// Refresh an RSS feed or folder.
    pub async fn refresh_item<T: AsRef<str> + Send + Sync>(&self, item_path: T) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            item_path: &'a str,
        }

        self.post_with(
            "rss/refreshItem",
            &Arg {
                item_path: item_path.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Create or replace an RSS auto-downloading rule.
    ///
    /// `rule_name` identifies the rule. `rule_definition` is JSON-encoded into
    /// qBittorrent's `ruleDef` request parameter.
    ///
    /// See qBittorrent's [Set auto-downloading rule](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#set-auto-downloading-rule)
    /// documentation.
    pub async fn set_rule(
        &self,
        rule_name: impl AsRef<str> + Send + Sync,
        rule_definition: impl Borrow<RssRuleDefinition> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            rule_name: &'a str,
            rule_def: String,
        }

        self.post_with(
            "rss/setRule",
            &Arg {
                rule_name: rule_name.as_ref(),
                rule_def: serde_json::to_string(rule_definition.borrow())?,
            },
        )
        .await?
        .end()
    }

    /// Rename an RSS auto-downloading rule.
    pub async fn rename_rule<T: AsRef<str> + Send + Sync>(
        &self,
        rule_name: T,
        new_rule_name: T,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            rule_name: &'a str,
            new_rule_name: &'a str,
        }

        self.post_with(
            "rss/renameRule",
            &Arg {
                rule_name: rule_name.as_ref(),
                new_rule_name: new_rule_name.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Remove an RSS auto-downloading rule.
    pub async fn remove_rule<T: AsRef<str> + Send + Sync>(&self, rule_name: T) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            rule_name: &'a str,
        }

        self.post_with(
            "rss/removeRule",
            &Arg {
                rule_name: rule_name.as_ref(),
            },
        )
        .await?
        .end()
    }

    /// Return all RSS auto-downloading rules keyed by rule name.
    ///
    /// See qBittorrent's [Get all auto-downloading rules](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-all-auto-downloading-rules)
    /// documentation.
    pub async fn get_rules(&self) -> Result<HashMap<String, RssRuleDefinition>> {
        self.get("rss/rules")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Return article titles matching an RSS auto-downloading rule, grouped by
    /// feed name.
    ///
    /// `rule_name` is the name of an existing rule whose filters qBittorrent
    /// applies to the configured feeds.
    ///
    /// See qBittorrent's [Get all articles matching a rule](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-all-articles-matching-a-rule)
    /// documentation.
    pub async fn get_matching_articles(
        &self,
        rule_name: impl AsRef<str> + Send + Sync,
    ) -> Result<HashMap<String, Vec<String>>> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            rule_name: &'a str,
        }

        self.get_with(
            "rss/matchingArticles",
            &Arg {
                rule_name: rule_name.as_ref(),
            },
        )
        .await?
        .json()
        .await
        .map_err(Into::into)
    }
}

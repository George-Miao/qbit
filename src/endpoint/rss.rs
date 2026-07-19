use serde::Serialize;

use crate::{Qbit, Result, ext::*};

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
}

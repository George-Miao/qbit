use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{ApiError, Qbit, Result, StatusCode, ext::*, model::*};

impl Qbit {
    /// Start a search using the selected plugins and category.
    pub async fn start_search(
        &self,
        pattern: impl AsRef<str> + Send + Sync,
        plugins: impl Into<Sep<String, '|'>> + Send + Sync,
        category: impl AsRef<str> + Send + Sync,
    ) -> Result<SearchJob> {
        #[derive(Serialize)]
        struct Arg<'a> {
            pattern: &'a str,
            plugins: String,
            category: &'a str,
        }

        self.post_with(
            "search/start",
            &Arg {
                pattern: pattern.as_ref(),
                plugins: plugins.into().to_string(),
                category: category.as_ref(),
            },
        )
        .await?
        .map_status(|status| {
            (status == StatusCode::CONFLICT).then_some(ApiError::SearchUnavailable.into())
        })?
        .json()
        .await
        .map_err(Into::into)
    }

    /// Stop the search job with the supplied ID.
    pub async fn stop_search(&self, id: i64) -> Result<()> {
        self.search_job_action("search/stop", id).await
    }

    /// Return the status of one search job or all search jobs.
    pub async fn get_search_status(
        &self,
        id: impl Into<Option<i64>> + Send + Sync,
    ) -> Result<Vec<SearchJobStatus>> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            id: Option<i64>,
        }

        self.get_with("search/status", &Arg { id: id.into() })
            .await?
            .map_status(search_not_found)?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Return a page of results for the supplied search job.
    pub async fn get_search_results(
        &self,
        id: i64,
        limit: impl Into<Option<i64>> + Send + Sync,
        offset: impl Into<Option<i64>> + Send + Sync,
    ) -> Result<SearchResults> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            id: i64,
            limit: Option<i64>,
            offset: Option<i64>,
        }

        self.get_with(
            "search/results",
            &Arg {
                id,
                limit: limit.into(),
                offset: offset.into(),
            },
        )
        .await?
        .map_status(|status| match status {
            StatusCode::NOT_FOUND => Some(ApiError::SearchJobNotFound.into()),
            StatusCode::CONFLICT => Some(ApiError::SearchInvalidOffset.into()),
            _ => None,
        })?
        .json()
        .await
        .map_err(Into::into)
    }

    /// Delete the search job with the supplied ID.
    pub async fn delete_search(&self, id: i64) -> Result<()> {
        self.search_job_action("search/delete", id).await
    }

    /// Return information about all installed search plugins.
    pub async fn get_search_plugins(&self) -> Result<Vec<SearchPlugin>> {
        self.get("search/plugins")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Install search plugins from URLs or local file paths.
    pub async fn install_search_plugins(
        &self,
        sources: impl Into<Sep<String, '|'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            sources: String,
        }

        self.post_with(
            "search/installPlugin",
            &Arg {
                sources: sources.into().to_string(),
            },
        )
        .await?
        .end()
    }

    /// Uninstall search plugins by name.
    pub async fn uninstall_search_plugins(
        &self,
        names: impl Into<Sep<String, '|'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            names: String,
        }

        self.post_with(
            "search/uninstallPlugin",
            &Arg {
                names: names.into().to_string(),
            },
        )
        .await?
        .end()
    }

    /// Enable or disable search plugins by name.
    pub async fn enable_search_plugins(
        &self,
        names: impl Into<Sep<String, '|'>> + Send + Sync,
        enable: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            names: String,
            enable: bool,
        }

        self.post_with(
            "search/enablePlugin",
            &Arg {
                names: names.into().to_string(),
                enable,
            },
        )
        .await?
        .end()
    }

    /// Check for updates to installed search plugins.
    pub async fn update_search_plugins(&self) -> Result<()> {
        self.post("search/updatePlugins").await?.end()
    }

    async fn search_job_action(&self, path: &'static str, id: i64) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            id: i64,
        }

        self.post_with(path, &Arg { id })
            .await?
            .map_status(search_not_found)?
            .end()
    }
}

fn search_not_found(status: StatusCode) -> Option<crate::Error> {
    (status == StatusCode::NOT_FOUND).then_some(ApiError::SearchJobNotFound.into())
}

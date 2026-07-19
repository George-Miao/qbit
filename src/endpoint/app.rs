use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{Error, Qbit, Result, ext::*, model::*};

impl Qbit {
    /// Return the qBittorrent application version.
    pub async fn get_version(&self) -> Result<String> {
        self.get("app/version")
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    /// Return the qBittorrent Web API version.
    pub async fn get_webapi_version(&self) -> Result<String> {
        self.get("app/webapiVersion")
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    /// Return qBittorrent build information and dependency versions.
    pub async fn get_build_info(&self) -> Result<BuildInfo> {
        self.get("app/buildInfo")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Get process info, including launch time.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.15.1).
    pub async fn get_process_info(&self) -> Result<ProcessInfo> {
        self.get("app/processInfo")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Get cookies stored in the qBittorrent WebUI.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.11.3).
    pub async fn get_cookies(&self) -> Result<Vec<CookieEntry>> {
        self.get("app/cookies")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Set cookies for the qBittorrent WebUI.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.11.3).
    pub async fn set_cookies(&self, cookies: &[SetCookieArg]) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            cookies: &'a str,
        }
        let json = serde_json::to_string(cookies)?;
        self.post_with("app/setCookies", &Arg { cookies: &json })
            .await?
            .end()
    }

    /// Shut down the qBittorrent application.
    pub async fn shutdown(&self) -> Result<()> {
        self.post("app/shutdown").await?.end()
    }

    /// Return the application preferences.
    pub async fn get_preferences(&self) -> Result<Preferences> {
        self.get("app/preferences")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Update the supplied application preferences.
    pub async fn set_preferences(
        &self,
        preferences: impl Borrow<Preferences> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            json: String,
        }

        self.post_with(
            "app/setPreferences",
            &Arg {
                json: serde_json::to_string(preferences.borrow())?,
            },
        )
        .await?
        .end()
    }

    /// Get free disk space at the given path (in bytes).
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.15.2).
    pub async fn get_free_space_at_path(
        &self,
        path: impl AsRef<Path> + Send + Sync,
    ) -> Result<u64> {
        #[derive(Serialize)]
        struct Arg<'a> {
            path: &'a Path,
        }
        self.get_with(
            "app/getFreeSpaceAtPathAction",
            &Arg {
                path: path.as_ref(),
            },
        )
        .await?
        .text()
        .await
        .map_err(Into::into)
        .and_then(|s| {
            s.parse::<u64>().map_err(|_| Error::BadResponse {
                explain: "getFreeSpaceAtPathAction returned non-numeric response",
            })
        })
    }

    /// Return the default path used to save torrent content.
    pub async fn get_default_save_path(&self) -> Result<PathBuf> {
        self.get("app/defaultSavePath")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .map(PathBuf::from)
    }
}

#![doc = include_str!("../README.md")]
#![warn(clippy::future_not_send)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

pub mod model;
pub use builder::QbitBuilder;
use bytes::Bytes;
use reqwest::{Client, Method, Response, StatusCode, header};
use serde::Serialize;
use serde_with::skip_serializing_none;
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};
use url::Url;

use crate::{ext::*, model::*};

mod builder;
mod ext;

#[derive(Clone)]
enum LoginState {
    CookieProvided {
        cookie: String,
    },
    NotLoggedIn {
        credential: Credential,
    },
    LoggedIn {
        cookie: String,
        credential: Credential,
    },
}

impl LoginState {
    fn as_cookie(&self) -> Option<&str> {
        match self {
            Self::CookieProvided { cookie } => Some(cookie),
            Self::NotLoggedIn { .. } => None,
            Self::LoggedIn { cookie, .. } => Some(cookie),
        }
    }

    fn as_credential(&self) -> Option<&Credential> {
        match self {
            Self::CookieProvided { .. } => None,
            Self::NotLoggedIn { credential } => Some(credential),
            Self::LoggedIn { credential, .. } => Some(credential),
        }
    }

    fn add_cookie(&mut self, cookie: String) {
        match self {
            Self::CookieProvided { .. } => {}
            Self::LoggedIn { credential, .. } | Self::NotLoggedIn { credential } => {
                *self = Self::LoggedIn {
                    cookie,
                    credential: credential.clone(),
                };
            }
        }
    }
}

/// Main entry point of the library. It provides a high-level API to interact
/// with qBittorrent WebUI API.
pub struct Qbit {
    client: Client,
    endpoint: Url,
    state: Mutex<LoginState>,
}

impl Qbit {
    /// Create a new [`QbitBuilder`] to build a [`Qbit`] instance.
    pub fn builder() -> QbitBuilder {
        QbitBuilder::new()
    }

    pub fn new_with_client<U>(endpoint: U, credential: Credential, client: Client) -> Self
    where
        U: TryInto<Url>,
        U::Error: Debug,
    {
        Self::builder()
            .endpoint(endpoint)
            .credential(credential)
            .client(client)
            .build()
    }

    pub fn new<U>(endpoint: U, credential: Credential) -> Self
    where
        U: TryInto<Url>,
        U::Error: Debug,
    {
        Self::new_with_client(endpoint, credential, Client::new())
    }

    #[deprecated = "Use `QbitBuilder::cookie` instead"]
    pub fn with_cookie(self, cookie: impl Into<String>) -> Self {
        Self {
            state: Mutex::from(LoginState::CookieProvided {
                cookie: cookie.into(),
            }),
            ..self
        }
    }

    pub async fn get_cookie(&self) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .as_cookie()
            .map(ToOwned::to_owned)
    }

    pub async fn logout(&self) -> Result<()> {
        self.get("auth/logout").await?.end()
    }

    pub async fn get_version(&self) -> Result<String> {
        self.get("app/version")
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    pub async fn get_webapi_version(&self) -> Result<String> {
        self.get("app/webapiVersion")
            .await?
            .text()
            .await
            .map_err(Into::into)
    }

    pub async fn get_build_info(&self) -> Result<BuildInfo> {
        self.get("app/buildInfo")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.post("app/shutdown", NONE).await?.end()
    }

    pub async fn get_preferences(&self) -> Result<Preferences> {
        self.get("app/preferences")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn set_preferences(
        &self,
        preferences: impl Borrow<Preferences> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            json: String,
        }

        self.post(
            "app/setPreferences",
            Some(&Arg {
                json: serde_json::to_string(preferences.borrow())?,
            }),
        )
        .await?
        .end()
    }

    pub async fn get_default_save_path(&self) -> Result<PathBuf> {
        self.get("app/defaultSavePath")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .map(PathBuf::from)
    }

    pub async fn get_logs(&self, arg: impl Borrow<GetLogsArg> + Send + Sync) -> Result<Vec<Log>> {
        self.get_with("log/main", arg.borrow())
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_peer_logs(
        &self,
        last_known_id: impl Into<Option<i64>> + Send + Sync,
    ) -> Result<Vec<PeerLog>> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            last_known_id: Option<i64>,
        }

        self.get_with(
            "log/peers",
            &Arg {
                last_known_id: last_known_id.into(),
            },
        )
        .await?
        .json()
        .await
        .map_err(Into::into)
    }

    pub async fn sync(&self, rid: impl Into<Option<i64>> + Send + Sync) -> Result<SyncData> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            rid: Option<i64>,
        }

        self.get_with("sync/maindata", &Arg { rid: rid.into() })
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_peers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        rid: impl Into<Option<i64>> + Send + Sync,
    ) -> Result<PeerSyncData> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hash: &'a str,
            rid: Option<i64>,
        }

        self.get_with(
            "sync/torrentPeers",
            &Arg {
                hash: hash.as_ref(),
                rid: rid.into(),
            },
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .json()
        .await
        .map_err(Into::into)
    }

    pub async fn get_transfer_info(&self) -> Result<TransferInfo> {
        self.get("transfer/info")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_speed_limits_mode(&self) -> Result<bool> {
        self.get("transfer/speedLimitsMode")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| match s.as_str() {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(Error::BadResponse {
                    explain: "Received non-number response body on `transfer/speedLimitsMode`",
                }),
            })
    }

    pub async fn toggle_speed_limits_mode(&self) -> Result<()> {
        self.post("transfer/toggleSpeedLimitsMode", None::<&()>)
            .await?
            .end()
    }

    pub async fn get_download_limit(&self) -> Result<u64> {
        self.get("transfer/downloadLimit")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| {
                s.parse().map_err(|_| Error::BadResponse {
                    explain: "Received non-number response body on `transfer/downloadLimit`",
                })
            })
    }

    pub async fn set_download_limit(&self, limit: u64) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            limit: u64,
        }

        self.post("transfer/setDownloadLimit", Some(&Arg { limit }))
            .await?
            .end()
    }

    pub async fn get_upload_limit(&self) -> Result<u64> {
        self.get("transfer/uploadLimit")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| {
                s.parse().map_err(|_| Error::BadResponse {
                    explain: "Received non-number response body on `transfer/uploadLimit`",
                })
            })
    }

    pub async fn set_upload_limit(&self, limit: u64) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            limit: u64,
        }

        self.post("transfer/setUploadLimit", Some(&Arg { limit }))
            .await?
            .end()
    }

    pub async fn ban_peers(&self, peers: impl Into<Sep<String, '|'>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            peers: String,
        }

        self.post(
            "transfer/banPeers",
            Some(&Arg {
                peers: peers.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn get_torrent_list(&self, arg: GetTorrentListArg) -> Result<Vec<Torrent>> {
        self.get_with("torrents/info", &arg)
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn export_torrent(&self, hash: impl AsRef<str> + Send + Sync) -> Result<Bytes> {
        self.get_with("torrents/export", &HashArg::new(hash.as_ref()))
            .await?
            .bytes()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_properties(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<TorrentProperty> {
        self.get_with("torrents/properties", &HashArg::new(hash.as_ref()))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_trackers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<Tracker>> {
        self.get_with("torrents/trackers", &HashArg::new(hash.as_ref()))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_web_seeds(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<WebSeed>> {
        self.get_with("torrents/webseeds", &HashArg::new(hash.as_ref()))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_contents(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        indexes: impl Into<Option<Sep<String, '|'>>> + Send + Sync,
    ) -> Result<Vec<TorrentContent>> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hash: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            indexes: Option<String>,
        }

        self.get_with(
            "torrents/files",
            &Arg {
                hash: hash.as_ref(),
                indexes: indexes.into().map(|s| s.to_string()),
            },
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .json()
        .await
        .map_err(Into::into)
    }

    pub async fn get_torrent_pieces_states(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<PieceState>> {
        self.get_with("torrents/pieceStates", &HashArg::new(hash.as_ref()))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_pieces_hashes(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<String>> {
        self.get_with("torrents/pieceHashes", &HashArg::new(hash.as_ref()))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn stop_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/stop", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn start_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/start", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn delete_torrents(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        delete_files: impl Into<Option<bool>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        #[serde(rename_all = "camelCase")]
        struct Arg {
            hashes: Hashes,
            delete_files: Option<bool>,
        }
        self.post(
            "torrents/delete",
            Some(&Arg {
                hashes: hashes.into(),
                delete_files: delete_files.into(),
            }),
        )
        .await?
        .end()
    }

    pub async fn recheck_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/recheck", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn reannounce_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/reannounce", Some(&HashesArg::new(hashes)))
            .await?
            .end()
    }

    pub async fn add_torrent(&self, arg: impl Borrow<AddTorrentArg> + Send + Sync) -> Result<()> {
        let a: &AddTorrentArg = arg.borrow();
        match &a.source {
            TorrentSource::Urls { urls: _ } => {
                self.post("torrents/add", Some(arg.borrow())).await?.end()
            }
            TorrentSource::TorrentFiles { torrents } => {
                for i in 0..3 {
                    // If it's not the first attempt, we need to re-login
                    self.login(i != 0).await?;
                    // Create a multipart form containing the torrent files and other arguments
                    let form = torrents.iter().fold(
                        serde_json::to_value(a)?
                            .as_object()
                            .unwrap()
                            .into_iter()
                            .fold(reqwest::multipart::Form::new(), |form, (k, v)| {
                                // If we directly call to_string() on a Value containing a string like "hello",
                                // it will include the quotes: "\"hello\"".
                                // We need to use as_str() first to get the inner string without quotes.
                                let v = match v.as_str() {
                                    Some(v_str) => v_str.to_string(),
                                    None => v.to_string(),
                                };
                                form.text(k.to_string(), v.to_string())
                            }),
                        |mut form, torrent| {
                            let p = reqwest::multipart::Part::bytes(torrent.data.clone())
                                .file_name(torrent.filename.to_string())
                                .mime_str("application/x-bittorrent")
                                .unwrap();
                            form = form.part("torrents", p);
                            form
                        },
                    );
                    let req = self
                        .client
                        .request(Method::POST, self.url("torrents/add"))
                        .multipart(form)
                        .header(header::COOKIE, {
                            self.state()
                                .as_cookie()
                                .expect("Cookie should be set after login")
                        });

                    trace!(request = ?req, "Sending request");
                    let res = req
                        .send()
                        .await?
                        .map_status(|code| match code as _ {
                            StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
                            _ => None,
                        })
                        .tap_ok(|response| trace!(?response));

                    match res {
                        Err(Error::ApiError(ApiError::NotLoggedIn)) => {
                            // Retry
                            warn!("Cookie is not valid, retrying");
                        }
                        Err(e) => return Err(e),
                        Ok(t) => return t.end(),
                    }
                }

                Err(Error::ApiError(ApiError::NotLoggedIn))
            }
        }
    }

    pub async fn add_trackers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        urls: impl Into<Sep<String, '\n'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hash: &'a str,
            urls: String,
        }

        self.post(
            "torrents/addTrackers",
            Some(&Arg {
                hash: hash.as_ref(),
                urls: urls.into().to_string(),
            }),
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .json()
        .await
        .map_err(Into::into)
    }

    pub async fn edit_trackers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        orig_url: Url,
        new_url: Url,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct EditTrackerArg<'a> {
            hash: &'a str,
            orig_url: Url,
            new_url: Url,
        }
        self.post(
            "torrents/editTracker",
            Some(&EditTrackerArg {
                hash: hash.as_ref(),
                orig_url,
                new_url,
            }),
        )
        .await?
        .map_status(|c| match c {
            StatusCode::BAD_REQUEST => Some(Error::ApiError(ApiError::InvalidTrackerUrl)),
            StatusCode::NOT_FOUND => Some(Error::ApiError(ApiError::TorrentNotFound)),
            StatusCode::CONFLICT => Some(Error::ApiError(ApiError::ConflictTrackerUrl)),
            _ => None,
        })?
        .end()
    }

    pub async fn remove_trackers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        urls: impl Into<Sep<Url, '|'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hash: &'a str,
            urls: Sep<Url, '|'>,
        }

        self.post(
            "torrents/removeTrackers",
            Some(&Arg {
                hash: hash.as_ref(),
                urls: urls.into(),
            }),
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .end()
    }

    pub async fn add_peers(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        peers: impl Into<Sep<String, '|'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct AddPeersArg {
            hash: String,
            peers: Sep<String, '|'>,
        }

        self.post(
            "torrents/addPeers",
            Some(&AddPeersArg {
                hash: hashes.into().to_string(),
                peers: peers.into(),
            }),
        )
        .await
        .and_then(|r| {
            r.map_status(|c| {
                if c == StatusCode::BAD_REQUEST {
                    Some(Error::ApiError(ApiError::InvalidPeers))
                } else {
                    None
                }
            })
        })?
        .end()
    }

    pub async fn increase_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/increasePrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::CONFLICT {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn decrease_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/decreasePrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::CONFLICT {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn maximal_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/topPrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::CONFLICT {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn minimal_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post("torrents/bottomPrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::CONFLICT {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn set_file_priority(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        indexes: impl Into<Sep<i64, '|'>> + Send + Sync,
        priority: Priority,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct SetFilePriorityArg<'a> {
            hash: &'a str,
            id: Sep<i64, '|'>,
            priority: Priority,
        }

        self.post(
            "torrents/filePrio",
            Some(&SetFilePriorityArg {
                hash: hash.as_ref(),
                id: indexes.into(),
                priority,
            }),
        )
        .await?
        .map_status(|c| match c {
            StatusCode::BAD_REQUEST => panic!("Invalid priority or id. This is a bug."),
            StatusCode::NOT_FOUND => Some(Error::ApiError(ApiError::TorrentNotFound)),
            StatusCode::CONFLICT => Some(Error::ApiError(ApiError::MetaNotDownloadedOrIdNotFound)),
            _ => None,
        })?;
        Ok(())
    }

    pub async fn get_torrent_download_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<HashMap<String, u64>> {
        self.get_with("torrents/downloadLimit", &HashesArg::new(hashes))
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn set_torrent_download_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        limit: u64,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            limit: u64,
        }

        self.post(
            "torrents/downloadLimit",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                limit,
            }),
        )
        .await?
        .end()
    }

    pub async fn set_torrent_shared_limit(
        &self,
        arg: impl Borrow<SetTorrentSharedLimitArg> + Send + Sync,
    ) -> Result<()> {
        self.post("torrents/setShareLimits", Some(arg.borrow()))
            .await?
            .end()
    }

    pub async fn get_torrent_upload_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<HashMap<String, u64>> {
        self.get_with("torrents/uploadLimit", &HashesArg::new(hashes))
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn set_torrent_upload_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        limit: u64,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            limit: u64,
        }

        self.post(
            "torrents/uploadLimit",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                limit,
            }),
        )
        .await?
        .end()
    }

    pub async fn set_torrent_location(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        location: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hashes: String,
            location: &'a Path,
        }

        self.post(
            "torrents/setLocation",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                location: location.as_ref(),
            }),
        )
        .await?
        .map_status(|c| match c {
            StatusCode::BAD_REQUEST => Some(Error::ApiError(ApiError::SavePathEmpty)),
            StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NoWriteAccess)),
            StatusCode::CONFLICT => Some(Error::ApiError(ApiError::UnableToCreateDir)),
            _ => None,
        })?
        .end()
    }

    pub async fn set_torrent_name<T: AsRef<str> + Send + Sync>(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        name: NonEmptyStr<T>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct RenameArg<'a> {
            hash: &'a str,
            name: &'a str,
        }

        self.post(
            "torrents/rename",
            Some(&RenameArg {
                hash: hash.as_ref(),
                name: name.as_str(),
            }),
        )
        .await?
        .map_status(|c| match c {
            StatusCode::NOT_FOUND => Some(Error::ApiError(ApiError::TorrentNotFound)),
            StatusCode::CONFLICT => panic!("Name should not be empty. This is a bug."),
            _ => None,
        })?
        .end()
    }

    pub async fn set_torrent_category(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        category: impl AsRef<str> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hashes: String,
            category: &'a str,
        }

        self.post(
            "torrents/setCategory",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                category: category.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::CONFLICT {
                Some(Error::ApiError(ApiError::CategoryNotFound))
            } else {
                None
            }
        })?
        .end()
    }

    pub async fn get_categories(&self) -> Result<HashMap<String, Category>> {
        self.get("torrents/categories")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn add_category<T: AsRef<str> + Send + Sync>(
        &self,
        category: NonEmptyStr<T>,
        save_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            category: &'a str,
            save_path: &'a Path,
        }

        self.post(
            "torrents/createCategory",
            Some(&Arg {
                category: category.as_str(),
                save_path: save_path.as_ref(),
            }),
        )
        .await?
        .end()
    }

    pub async fn edit_category<T: AsRef<str> + Send + Sync>(
        &self,
        category: NonEmptyStr<T>,
        save_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            category: &'a str,
            save_path: &'a Path,
        }

        self.post(
            "torrents/createCategory",
            Some(&Arg {
                category: category.as_str(),
                save_path: save_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::CONFLICT {
                Some(Error::ApiError(ApiError::CategoryEditingFailed))
            } else {
                None
            }
        })?
        .end()
    }

    pub async fn remove_categories(
        &self,
        categories: impl Into<Sep<String, '\n'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            categories: &'a str,
        }

        self.post(
            "torrents/removeCategories",
            Some(&Arg {
                categories: &categories.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn add_torrent_tags(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        tags: impl Into<Sep<String, '\n'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hashes: String,
            tags: &'a str,
        }

        self.post(
            "torrents/addTags",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                tags: &tags.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn remove_torrent_tags(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        tags: Option<impl Into<Sep<String, ','>> + Send>,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            hashes: String,
            tags: Option<String>,
        }

        self.post(
            "torrents/removeTags",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                tags: tags.map(|t| t.into().to_string()),
            }),
        )
        .await?
        .end()
    }

    pub async fn get_all_tags(&self) -> Result<Vec<String>> {
        self.get("torrents/tags")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    pub async fn create_tags(&self, tags: impl Into<Sep<String, ','>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            tags: String,
        }

        self.post(
            "torrents/createTags",
            Some(&Arg {
                tags: tags.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn delete_tags(&self, tags: impl Into<Sep<String, ','>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            tags: String,
        }

        self.post(
            "torrents/deleteTags",
            Some(&Arg {
                tags: tags.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn set_auto_management(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        enable: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            enable: bool,
        }

        self.post(
            "torrents/setAutoManagement",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                enable,
            }),
        )
        .await?
        .end()
    }

    pub async fn toggle_sequential_download(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post(
            "torrents/toggleSequentialDownload",
            Some(&HashesArg::new(hashes)),
        )
        .await?
        .end()
    }

    pub async fn toggle_first_last_piece_priority(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post(
            "torrents/toggleFirstLastPiecePrio",
            Some(&HashesArg::new(hashes)),
        )
        .await?
        .end()
    }

    pub async fn set_force_start(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        value: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            value: bool,
        }

        self.post(
            "torrents/setForceStart",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                value,
            }),
        )
        .await?
        .end()
    }

    pub async fn set_super_seeding(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        value: bool,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            value: bool,
        }

        self.post(
            "torrents/setSuperSeeding",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                value,
            }),
        )
        .await?
        .end()
    }

    pub async fn rename_file(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        old_path: impl AsRef<Path> + Send + Sync,
        new_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            hash: &'a str,
            old_path: &'a Path,
            new_path: &'a Path,
        }

        self.post(
            "torrents/renameFile",
            Some(&Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::CONFLICT {
                Error::ApiError(ApiError::InvalidPath).pipe(Some)
            } else {
                None
            }
        })?
        .end()
    }

    pub async fn rename_folder(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        old_path: impl AsRef<Path> + Send + Sync,
        new_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            hash: &'a str,
            old_path: &'a Path,
            new_path: &'a Path,
        }

        self.post(
            "torrents/renameFolder",
            Some(&Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::CONFLICT {
                Error::ApiError(ApiError::InvalidPath).pipe(Some)
            } else {
                None
            }
        })?
        .end()
    }

    pub async fn add_folder<T: AsRef<str> + Send + Sync>(&self, path: T) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            path: &'a str,
        }

        self.post(
            "rss/addFolder",
            Some(&Arg {
                path: path.as_ref(),
            }),
        )
        .await?
        .end()
    }

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

        self.post(
            "rss/addFeed",
            Some(&Arg {
                url: url.as_ref(),
                path: path.as_ref().map(AsRef::as_ref),
            }),
        )
        .await?
        .end()
    }

    pub async fn remove_item<T: AsRef<str> + Send + Sync>(&self, path: T) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            path: &'a str,
        }

        self.post(
            "rss/removeItem",
            Some(&Arg {
                path: path.as_ref(),
            }),
        )
        .await?
        .end()
    }

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

        self.post(
            "rss/moveItem",
            Some(&Arg {
                item_path: item_path.as_ref(),
                dest_path: dest_path.as_ref(),
            }),
        )
        .await?
        .end()
    }

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

        self.post(
            "rss/markAsRead",
            Some(&Arg {
                item_path: item_path.as_ref(),
                article_id: article_id.as_ref().map(AsRef::as_ref),
            }),
        )
        .await?
        .end()
    }

    pub async fn refresh_item<T: AsRef<str> + Send + Sync>(&self, item_path: T) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            item_path: &'a str,
        }

        self.post(
            "rss/refreshItem",
            Some(&Arg {
                item_path: item_path.as_ref(),
            }),
        )
        .await?
        .end()
    }

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

        self.post(
            "rss/renameRule",
            Some(&Arg {
                rule_name: rule_name.as_ref(),
                new_rule_name: new_rule_name.as_ref(),
            }),
        )
        .await?
        .end()
    }

    pub async fn remove_rule<T: AsRef<str> + Send + Sync>(&self, rule_name: T) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Arg<'a> {
            rule_name: &'a str,
        }

        self.post(
            "rss/removeRule",
            Some(&Arg {
                rule_name: rule_name.as_ref(),
            }),
        )
        .await?
        .end()
    }

    fn url(&self, path: &'static str) -> Url {
        self.endpoint
            .join("api/v2/")
            .unwrap()
            .join(path)
            .expect("Invalid API endpoint")
    }

    fn state(&self) -> MutexGuard<'_, LoginState> {
        self.state.lock().unwrap()
    }

    /// Log in to qBittorrent. Set force to `true` to forcefully re-login
    /// regardless if cookie is already set.
    pub async fn login(&self, force: bool) -> Result<()> {
        let re_login = force || { self.state().as_cookie().is_none() };
        if re_login {
            debug!("Cookie not found, logging in");
            self.client
                .request(Method::POST, self.url("auth/login"))
                .pipe(|req| {
                    req.form(
                        self.state()
                            .as_credential()
                            .expect("Credential should be set if cookie is not set"),
                    )
                })
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::IpBanned)),
                    _ => None,
                })?
                .extract::<Cookie>()?
                .pipe(|Cookie(cookie)| self.state.lock().unwrap().add_cookie(cookie));

            debug!("Log in success");
        } else {
            trace!("Already logged in, skipping");
        }

        Ok(())
    }

    async fn request(
        &self,
        method: Method,
        path: &'static str,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        for i in 0..3 {
            // If it's not the first attempt, we need to re-login
            self.login(i != 0).await?;

            let mut req =
                self.client
                    .request(method.clone(), self.url(path))
                    .header(header::COOKIE, {
                        self.state()
                            .as_cookie()
                            .expect("Cookie should be set after login")
                    });

            if let Some(ref body) = body {
                match method {
                    Method::GET => req = req.query(body),
                    Method::POST => req = req.form(body),
                    _ => unreachable!("Only GET and POST are supported"),
                }
            }
            trace!(request = ?req, "Sending request");
            let res = req
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
                    _ => None,
                })
                .tap_ok(|response| trace!(?response));

            match res {
                Err(Error::ApiError(ApiError::NotLoggedIn)) => {
                    // Retry
                    warn!("Cookie is not valid, retrying");
                }
                Err(e) => return Err(e),
                Ok(t) => return Ok(t),
            }
        }

        Err(Error::ApiError(ApiError::NotLoggedIn))
    }

    async fn get(&self, path: &'static str) -> Result<Response> {
        self.request(Method::GET, path, NONE).await
    }

    async fn get_with(
        &self,
        path: &'static str,
        param: &(impl Serialize + Sync),
    ) -> Result<Response> {
        self.request(Method::GET, path, Some(param)).await
    }

    async fn post(
        &self,
        path: &'static str,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        self.request(Method::POST, path, body).await
    }
}

impl Clone for Qbit {
    fn clone(&self) -> Self {
        let state = self.state.lock().unwrap().clone();
        Self {
            client: self.client.clone(),
            endpoint: self.endpoint.clone(),
            state: Mutex::new(state),
        }
    }
}

const NONE: Option<&'static ()> = Option::None;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API Returned bad response: {explain}")]
    BadResponse { explain: &'static str },

    #[error("API returned unknown status code: {0}")]
    UnknownHttpCode(StatusCode),

    #[error("Non ASCII header")]
    NonAsciiHeader,

    #[error(transparent)]
    ApiError(#[from] ApiError),

    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

/// Errors defined and returned by the API
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User's IP is banned for too many failed login attempts")]
    IpBanned,

    #[error("API routes requires login, try again")]
    NotLoggedIn,

    #[error("Torrent not found")]
    TorrentNotFound,

    #[error("Torrent name is empty")]
    TorrentNameEmpty,

    #[error("`newUrl` is not a valid URL")]
    InvalidTrackerUrl,

    #[error("`newUrl` already exists for the torrent or `origUrl` was not found")]
    ConflictTrackerUrl,

    #[error("None of the given peers are valid")]
    InvalidPeers,

    #[error("Torrent queueing is not enabled")]
    QueueingDisabled,

    #[error("Torrent metadata hasn't downloaded yet or at least one file id was not found")]
    MetaNotDownloadedOrIdNotFound,

    #[error("Save path is empty")]
    SavePathEmpty,

    #[error("User does not have write access to the directory")]
    NoWriteAccess,

    #[error("Unable to create save path directory")]
    UnableToCreateDir,

    #[error("Category name does not exist")]
    CategoryNotFound,

    #[error("Category editing failed")]
    CategoryEditingFailed,

    #[error("Invalid `newPath` or `oldPath`, or `newPath` already in use")]
    InvalidPath,
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod test {
    use std::{
        env,
        ops::Deref,
        sync::{LazyLock, OnceLock},
    };

    use tracing::info;

    use super::*;

    async fn prepare<'a>() -> Result<&'a Qbit> {
        static PREPARE: LazyLock<(Credential, Url)> = LazyLock::new(|| {
            dotenv::dotenv().expect("Failed to load .env file");
            tracing_subscriber::fmt::init();

            (
                Credential::new(
                    env::var("QBIT_USERNAME").expect("QBIT_USERNAME not set"),
                    env::var("QBIT_PASSWORD").expect("QBIT_PASSWORD not set"),
                ),
                env::var("QBIT_BASEURL")
                    .expect("QBIT_BASEURL not set")
                    .parse()
                    .expect("QBIT_BASEURL is not a valid url"),
            )
        });
        static API: OnceLock<Qbit> = OnceLock::new();

        if let Some(api) = API.get() {
            Ok(api)
        } else {
            let (credential, url) = PREPARE.deref().clone();
            let api = Qbit::new(url, credential);
            api.login(false).await?;
            drop(API.set(api));
            Ok(API.get().unwrap())
        }
    }

    #[tokio::test]
    async fn test_login() {
        let client = prepare().await.unwrap();

        info!(
            version = client.get_version().await.unwrap(),
            "Login success"
        );
    }

    #[tokio::test]
    async fn test_preference() {
        let client = prepare().await.unwrap();

        client.get_preferences().await.unwrap();
    }

    #[tokio::test]
    async fn test_add_torrent() {
        let client = prepare().await.unwrap();
        let arg = AddTorrentArg {
            source: TorrentSource::Urls {
                urls: vec![
                    "https://releases.ubuntu.com/22.04/ubuntu-22.04.4-desktop-amd64.iso.torrent"
                        .parse()
                        .unwrap(),
                ]
                .into(),
            },
            ratio_limit: Some(1.0),
            ..AddTorrentArg::default()
        };
        client.add_torrent(arg).await.unwrap();
    }
    #[tokio::test]
    async fn test_add_torrent_file() {
        let client = prepare().await.unwrap();
        let arg = AddTorrentArg {
            source: TorrentSource::TorrentFiles {
                torrents: vec![ TorrentFile {
                    filename: "ubuntu-22.04.4-desktop-amd64.iso.torrent".into(),
                    data: reqwest::get("https://releases.ubuntu.com/22.04/ubuntu-22.04.4-desktop-amd64.iso.torrent")
                        .await
                        .unwrap()
                        .bytes()
                        .await
                        .unwrap()
                        .to_vec(),
                }]
            },
            ratio_limit: Some(1.0),
            ..AddTorrentArg::default()
        };
        client.add_torrent(arg).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_torrent_list() {
        let client = prepare().await.unwrap();
        let list = client
            .get_torrent_list(GetTorrentListArg::default())
            .await
            .unwrap();
        print!("{:#?}", list);
    }
}

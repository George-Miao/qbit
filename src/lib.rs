#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

mod client;
use client::*;

pub mod model;
pub use builder::QbitBuilder;
use bytes::Bytes;
use serde::Serialize;
use serde_with::skip_serializing_none;
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};
use url::Url;

use crate::{ext::*, model::*};

mod builder;
mod ext;

#[derive(Clone, Debug)]
enum LoginState {
    CookieProvided {
        cookie: String,
    },
    ApiKeyProvided {
        api_key: String,
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
    fn as_header(&self) -> (Option<header::HeaderName>, Option<&str>) {
        match self {
            Self::CookieProvided { cookie } => (Some(header::COOKIE), Some(cookie)),
            Self::ApiKeyProvided { api_key } => (Some(header::AUTHORIZATION), Some(api_key)),
            Self::NotLoggedIn { .. } => (None, None),
            Self::LoggedIn { cookie, .. } => (Some(header::COOKIE), Some(cookie)),
        }
    }

    fn as_credential(&self) -> Option<&Credential> {
        match self {
            Self::CookieProvided { .. } => None,
            Self::ApiKeyProvided { .. } => None,
            Self::NotLoggedIn { credential } => Some(credential),
            Self::LoggedIn { credential, .. } => Some(credential),
        }
    }

    fn add_cookie(&mut self, cookie: String) {
        match self {
            Self::CookieProvided { .. } => {}
            Self::ApiKeyProvided { .. } => {}
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

    pub async fn get_cookie(&self) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .as_header().1
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
    pub async fn shutdown(&self) -> Result<()> {
        self.post("app/shutdown").await?.end()
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
        self.get_with("app/getFreeSpaceAtPathAction", &Arg { path: path.as_ref() })
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
        self.post("transfer/toggleSpeedLimitsMode").await?.end()
    }

    /// Get global and alternative speed limits (KiB/s, -1 = unlimited).
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.16.0).
    pub async fn get_speed_limits(&self) -> Result<SpeedLimits> {
        self.get("transfer/getSpeedLimits")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Set global and alternative speed limits (KiB/s, -1 = unlimited).
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.16.0).
    pub async fn set_speed_limits(&self, limits: &SpeedLimits) -> Result<()> {
        self.post_with("transfer/setSpeedLimits", limits)
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

        self.post_with("transfer/setDownloadLimit", &Arg { limit })
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

        self.post_with("transfer/setUploadLimit", &Arg { limit })
            .await?
            .end()
    }

    pub async fn ban_peers(&self, peers: impl Into<Sep<String, '|'>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            peers: String,
        }

        self.post_with(
            "transfer/banPeers",
            &Arg {
                peers: peers.into().to_string(),
            },
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

    /// Download a completed file from a torrent's content.
    ///
    /// `file` can be either a file index (as a number) or a path relative
    /// to the torrent content root.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.16.0).
    pub async fn download_torrent_file(
        &self,
        hash: impl AsRef<str> + Send + Sync,
        file: impl AsRef<str> + Send + Sync,
    ) -> Result<Bytes> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hash: &'a str,
            file: &'a str,
        }
        self.post_with(
            "torrents/downloadFile",
            &Arg {
                hash: hash.as_ref(),
                file: file.as_ref(),
            },
        )
        .await?
        .map_status(|c| match c {
            StatusCode::NOT_FOUND => Some(Error::ApiError(ApiError::TorrentNotFound)),
            StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
            _ => None,
        })?
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

    /// Get the availability (number of distributed copies) of each piece
    /// of a torrent. Returns a vector where each element is the availability
    /// count for the corresponding piece index.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.15.1).
    pub async fn get_torrent_piece_availability(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<i64>> {
        self.get_with("torrents/pieceAvailability", &HashArg::new(hash.as_ref()))
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
        self.post_with("torrents/stop", &HashesArg::new(hashes))
            .await?
            .end()
    }

    pub async fn start_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/start", &HashesArg::new(hashes))
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
        self.post_with(
            "torrents/delete",
            &Arg {
                hashes: hashes.into(),
                delete_files: delete_files.into(),
            },
        )
        .await?
        .end()
    }

    pub async fn recheck_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/recheck", &HashesArg::new(hashes))
            .await?
            .end()
    }

    pub async fn reannounce_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/reannounce", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Reannounce torrents, optionally specifying which trackers to contact.
    ///
    /// `trackers` is a pipe-separated list of tracker URLs. Added in
    /// qBittorrent 5.2.0 (Web API v2.11.10).
    pub async fn reannounce_torrents_with_trackers(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        trackers: impl Into<Sep<String, '|'>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            hashes: String,
            trackers: String,
        }
        self.post_with(
            "torrents/reannounce",
            &Arg {
                hashes: hashes.into().to_string(),
                trackers: trackers.into().to_string(),
            },
        )
        .await?
        .end()
    }

    pub async fn add_torrent(&self, arg: impl Borrow<AddTorrentArg> + Send + Sync) -> Result<()> {
        use multipart::Form;

        fn make_form(
            arg: &AddTorrentArg,
            torrents: &[TorrentFile],
        ) -> Result<Form, serde_json::Error> {
            let form = serde_json::to_value(arg)?
                .as_object()
                .unwrap()
                .into_iter()
                .fold(Form::new(), |form, (k, v)| {
                    let v = match v.as_str() {
                        Some(v_str) => v_str.to_string(),
                        None => v.to_string(),
                    };
                    form.text(k.to_string(), v.to_string())
                });

            torrents
                .iter()
                .fold(form, |mut form, torrent| {
                    let p = multipart::Part::bytes(torrent.data.clone())
                        .file_name(torrent.filename.to_string())
                        .mime_str("application/x-bittorrent")
                        .unwrap();
                    form = form.part("torrents", p);
                    form
                })
                .pipe(Ok)
        }

        let args: &AddTorrentArg = arg.borrow();

        // qBittorrent 5.0+ renamed the `paused` field to `stopped`. Mirror
        // whichever one the caller set into the other so the request is
        // honored regardless of server version.
        // See <https://github.com/George-Miao/qbit/issues/40>.
        let owned;
        let args = if args.paused.is_some() ^ args.stopped.is_some() {
            let mut args = args.clone();
            if args.paused.is_none() {
                args.paused = args.stopped.clone();
            } else {
                args.stopped = args.paused.clone();
            }
            owned = args;
            &owned
        } else {
            args
        };

        match &args.source {
            TorrentSource::Urls { urls: _ } => {
                self.post_with("torrents/add", args).await?.end()
            }
            TorrentSource::TorrentFiles { torrents } => self
                .request(
                    Method::POST,
                    "torrents/add",
                    Some(|req: RequestBuilder| req.multipart(make_form(args, torrents)?).check()),
                )
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
                    StatusCode::UNSUPPORTED_MEDIA_TYPE => {
                        Some(Error::ApiError(ApiError::TorrentFileInvalid))
                    }
                    // qBittorrent 5.2.0+: 409 = all torrents failed (e.g. duplicate)
                    StatusCode::CONFLICT => {
                        Some(Error::ApiError(ApiError::TorrentAddFailed))
                    }
                    _ => None,
                })?
                .end(),
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

        self.post_with(
            "torrents/addTrackers",
            &Arg {
                hash: hash.as_ref(),
                urls: urls.into().to_string(),
            },
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
        self.post_with(
            "torrents/editTracker",
            &EditTrackerArg {
                hash: hash.as_ref(),
                orig_url,
                new_url,
            },
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

        self.post_with(
            "torrents/removeTrackers",
            &Arg {
                hash: hash.as_ref(),
                urls: urls.into(),
            },
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

        self.post_with(
            "torrents/addPeers",
            &AddPeersArg {
                hash: hashes.into().to_string(),
                peers: peers.into(),
            },
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
        self.post_with("torrents/increasePrio", &HashesArg::new(hashes))
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
        self.post_with("torrents/decreasePrio", &HashesArg::new(hashes))
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
        self.post_with("torrents/topPrio", &HashesArg::new(hashes))
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
        self.post_with("torrents/bottomPrio", &HashesArg::new(hashes))
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

        self.post_with(
            "torrents/filePrio",
            &SetFilePriorityArg {
                hash: hash.as_ref(),
                id: indexes.into(),
                priority,
            },
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

        self.post_with(
            "torrents/downloadLimit",
            &Arg {
                hashes: hashes.into().to_string(),
                limit,
            },
        )
        .await?
        .end()
    }

    pub async fn set_torrent_shared_limit(
        &self,
        arg: impl Borrow<SetTorrentSharedLimitArg> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/setShareLimits", arg.borrow())
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

        self.post_with(
            "torrents/uploadLimit",
            &Arg {
                hashes: hashes.into().to_string(),
                limit,
            },
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

        self.post_with(
            "torrents/setLocation",
            &Arg {
                hashes: hashes.into().to_string(),
                location: location.as_ref(),
            },
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

        self.post_with(
            "torrents/rename",
            &RenameArg {
                hash: hash.as_ref(),
                name: name.as_str(),
            },
        )
        .await?
        .map_status(|c| match c {
            StatusCode::NOT_FOUND => Some(Error::ApiError(ApiError::TorrentNotFound)),
            StatusCode::CONFLICT => panic!("Name should not be empty. This is a bug."),
            _ => None,
        })?
        .end()
    }

    /// Set the comment for one or more torrents.
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.12.1).
    pub async fn set_torrent_comment(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        comment: &str,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            hashes: String,
            comment: &'a str,
        }

        self.post_with(
            "torrents/setComment",
            &Arg {
                hashes: hashes.into().to_string(),
                comment,
            },
        )
        .await?
        .map_status(|c| match c {
            StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::NotLoggedIn)),
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

        self.post_with(
            "torrents/setCategory",
            &Arg {
                hashes: hashes.into().to_string(),
                category: category.as_ref(),
            },
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

        self.post_with(
            "torrents/createCategory",
            &Arg {
                category: category.as_str(),
                save_path: save_path.as_ref(),
            },
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

        self.post_with(
            "torrents/createCategory",
            &Arg {
                category: category.as_str(),
                save_path: save_path.as_ref(),
            },
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

        self.post_with(
            "torrents/removeCategories",
            &Arg {
                categories: &categories.into().to_string(),
            },
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

        self.post_with(
            "torrents/addTags",
            &Arg {
                hashes: hashes.into().to_string(),
                tags: &tags.into().to_string(),
            },
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

        self.post_with(
            "torrents/removeTags",
            &Arg {
                hashes: hashes.into().to_string(),
                tags: tags.map(|t| t.into().to_string()),
            },
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

        self.post_with(
            "torrents/createTags",
            &Arg {
                tags: tags.into().to_string(),
            },
        )
        .await?
        .end()
    }

    pub async fn delete_tags(&self, tags: impl Into<Sep<String, ','>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            tags: String,
        }

        self.post_with(
            "torrents/deleteTags",
            &Arg {
                tags: tags.into().to_string(),
            },
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

        self.post_with(
            "torrents/setAutoManagement",
            &Arg {
                hashes: hashes.into().to_string(),
                enable,
            },
        )
        .await?
        .end()
    }

    pub async fn toggle_sequential_download(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/toggleSequentialDownload", &HashesArg::new(hashes))
            .await?
            .end()
    }

    pub async fn toggle_first_last_piece_priority(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/toggleFirstLastPiecePrio", &HashesArg::new(hashes))
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

        self.post_with(
            "torrents/setForceStart",
            &Arg {
                hashes: hashes.into().to_string(),
                value,
            },
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

        self.post_with(
            "torrents/setSuperSeeding",
            &Arg {
                hashes: hashes.into().to_string(),
                value,
            },
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

        self.post_with(
            "torrents/renameFile",
            &Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            },
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

        self.post_with(
            "torrents/renameFolder",
            &Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            },
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

        self.post_with(
            "rss/addFolder",
            &Arg {
                path: path.as_ref(),
            },
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

    /// Log in to qBittorrent.
    ///
    /// Set force to `true` to force re-login regardless if cookie is already
    /// set.
    pub async fn login(&self, force: bool) -> Result<()> {
        let re_login = force || { self.state().as_header().1.is_none() };
        if re_login {
            let credential = match self.state().as_credential() {
                Some(credential) => credential.clone(),
                None => {
                    trace!("API key or cookie auth in use, skipping login");
                    return Ok(());
                }
            };
            debug!("Cookie not found, logging in");
            self.client
                .request(Method::POST, self.url("auth/login"))
                .check()?
                .pipe(|req| req.form(&credential))
                .check()?
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::IpBanned)),
                    StatusCode::UNAUTHORIZED => Some(Error::ApiError(ApiError::BadCredentials)),
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

    async fn get(&self, path: &'static str) -> Result<Response> {
        self.request(Method::GET, path, NONE).await
    }

    async fn get_with(
        &self,
        path: &'static str,
        param: &(impl Serialize + Sync),
    ) -> Result<Response> {
        self.request(
            Method::GET,
            path,
            Some(|req: RequestBuilder| req.query(param).check()),
        )
        .await
    }

    async fn post(&self, path: &'static str) -> Result<Response> {
        self.request(Method::POST, path, NONE).await
    }

    async fn post_with(
        &self,
        path: &'static str,
        body: &(impl Serialize + Sync),
    ) -> Result<Response> {
        self.request(
            Method::POST,
            path,
            Some(|req: RequestBuilder| req.form(body).check()),
        )
        .await
    }

    async fn request(
        &self,
        method: Method,
        path: &'static str,
        mut map: Option<impl FnMut(RequestBuilder) -> Result<RequestBuilder>>,
    ) -> Result<Response> {
        for i in 0..3 {
            // If it's not the first attempt, we need to re-login
            self.login(i != 0).await?;

            let (header_key, header_value) = {
                let state = self.state();
                let (header_key, header_value) = state.as_header();
                let header_key = header_key.expect("Should always have header key if logged in");
                let header_value = header_value.expect("Should always have header value if logged in");
                (header_key.to_owned(), header_value.to_owned())
            };
            
            let mut req = self
                .client
                .request(method.clone(), self.url(path))
                .check()?
                .header(header_key, header_value)
                .check()?;

            if let Some(map) = map.as_mut() {
                req = map(req)?;
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

const NONE: Option<fn(RequestBuilder) -> Result<RequestBuilder>> = Option::None;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http error: {0}")]
    HttpError(#[from] client::Error),

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
    /// API returned 401 - invalid credentials
    #[error("Invalid credentials")]
    BadCredentials,

    /// User's IP is banned for too many failed login attempts
    #[error("User's IP is banned for too many failed login attempts")]
    IpBanned,

    /// API routes requires login, try again
    #[error("API routes requires login, try again")]
    NotLoggedIn,

    /// Torrent not found
    #[error("Torrent not found")]
    TorrentNotFound,

    /// Torrent name is empty
    #[error("Torrent name is empty")]
    TorrentNameEmpty,

    /// Torrent file is not valid
    #[error("Torrent file is not valid")]
    TorrentFileInvalid,

    /// Torrent could not be added (e.g. duplicate)
    #[error("Torrent could not be added")]
    TorrentAddFailed,

    /// `newUrl` is not a valid URL
    #[error("`newUrl` is not a valid URL")]
    InvalidTrackerUrl,

    /// `newUrl` already exists for the torrent or `origUrl` was not found
    #[error("`newUrl` already exists for the torrent or `origUrl` was not found")]
    ConflictTrackerUrl,

    /// None of the given peers are valid
    #[error("None of the given peers are valid")]
    InvalidPeers,

    /// Torrent queueing is not enabled
    #[error("Torrent queueing is not enabled")]
    QueueingDisabled,

    /// Torrent metadata hasn't downloaded yet or at least one file id was not
    /// found
    #[error("Torrent metadata hasn't downloaded yet or at least one file id was not found")]
    MetaNotDownloadedOrIdNotFound,

    /// Save path is empty
    #[error("Save path is empty")]
    SavePathEmpty,

    /// User does not have write access to the directory
    #[error("User does not have write access to the directory")]
    NoWriteAccess,

    /// Unable to create save path directory
    #[error("Unable to create save path directory")]
    UnableToCreateDir,

    /// Category name does not exist
    #[error("Category name does not exist")]
    CategoryNotFound,

    /// Category editing failed
    #[error("Category editing failed")]
    CategoryEditingFailed,

    /// Invalid `newPath` or `oldPath`, or `newPath` already in use
    #[error("Invalid `newPath` or `oldPath`, or `newPath` already in use")]
    InvalidPath,
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod test {
    use std::{
        env, ops::Deref, sync::{LazyLock, Once, OnceLock},
    };

    use tracing::info;

    use super::*;

    #[cfg(feature = "reqwest")]
    async fn sleep(duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }

    #[cfg(feature = "cyper")]
    async fn sleep(duration: std::time::Duration) {
        compio::time::sleep(duration).await;
    }

    async fn init()  {
        static INIT: Once = Once::new();

        INIT.call_once(|| {
            dotenv::dotenv().expect("Failed to load .env file");
            tracing_subscriber::fmt::init();
        });
    }

    async fn client_with_credentials<'a>() -> Result<&'a Qbit> {
        init().await;
        static PREPARE: LazyLock<(Credential, Url)> = LazyLock::new(|| {
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

    async fn client_with_api_key<'a>() -> Result<&'a Qbit> {
        init().await;
        static PREPARE: LazyLock<Option<(String, Url)>> = LazyLock::new(|| {
            let api_key = env::var("QBIT_API_KEY").ok()?;
            let url = env::var("QBIT_BASEURL")
                .expect("QBIT_BASEURL not set")
                .parse()
                .expect("QBIT_BASEURL is not a valid url");
            Some((api_key, url))
        });
        static API: OnceLock<Option<Qbit>> = OnceLock::new();

        let prepared = PREPARE.deref();
        let Some((api_key, url)) = prepared.clone() else {
            return Err(Error::ApiError(ApiError::NotLoggedIn));
        };

        if let Some(Some(api)) = API.get() {
            Ok(api)
        } else {
            let api = Qbit::builder()
                .endpoint(url)
                .api_key(api_key)
                .build();
            drop(API.set(Some(api)));
            Ok(API.get().unwrap().as_ref().unwrap())
        }
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_login() {
        let client = client_with_credentials().await.unwrap();

        info!(
            version = client.get_version().await.unwrap(),
            "Login success"
        );
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_login_bad_credentials() {
        init().await;
        let url: Url = env::var("QBIT_BASEURL")
            .expect("QBIT_BASEURL not set")
            .parse()
            .expect("QBIT_BASEURL is not a valid url");
        let client = Qbit::new(url, Credential::new("no_such_user", "wrong_password"));
        let err = client.login(true).await.unwrap_err();
        assert!(matches!(err, Error::ApiError(ApiError::BadCredentials)));
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_version_api_key() {
        let client = match client_with_api_key().await {
            Ok(c) => c,
            Err(_) => {
                eprintln!("QBIT_API_KEY not set, skipping API key test");
                return;
            }
        };

        info!(
            version = client.get_version().await.unwrap(),
            "Login success"
        );
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_preference() {
        let client = client_with_credentials().await.unwrap();

        client.get_preferences().await.unwrap();
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_add_torrent() {
        let client = client_with_credentials().await.unwrap();
        let arg = AddTorrentArg {
            source: TorrentSource::Urls {
                urls: vec![
                    "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.torrent"
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
    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_add_torrent_file() {
        let client = client_with_credentials().await.unwrap();
        let arg = AddTorrentArg {
            source: TorrentSource::TorrentFiles {
                torrents: vec![ TorrentFile {
                    filename: "leaves.torrent".into(),
                    data: client::get("https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/leaves.torrent")
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

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_get_torrent_list() {
        let client = client_with_credentials().await.unwrap();
        let list = client
            .get_torrent_list(GetTorrentListArg::default())
            .await
            .unwrap();
        print!("{:#?}", list);
    }

    #[cfg_attr(feature = "reqwest", tokio::test)]
    #[cfg_attr(feature = "cyper", compio::test)]
    async fn test_download_torrent_file() {
        let client = client_with_credentials().await.unwrap();
        let expected = client::get(
            "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.txt",
        )
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
        let arg = AddTorrentArg {
            source: TorrentSource::Urls {
                urls: vec![
                    "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.torrent"
                        .parse()
                        .unwrap(),
                ]
                .into(),
            },
            ..AddTorrentArg::default()
        };
        client.add_torrent(arg).await.unwrap();
        let mut hash = None;
        for _ in 0..30 {
            let list = client
                .get_torrent_list(GetTorrentListArg::default())
                .await
                .unwrap();
            hash = list
                .iter()
                .find(|torrent| torrent.name.as_deref() == Some("alice.txt"))
                .and_then(|torrent| torrent.hash.clone());
            if hash.is_some() {
                break;
            }
            sleep(std::time::Duration::from_secs(1)).await;
        }
        let hash = hash.expect("alice torrent was not added in time");

        // Wait for the torrent to finish downloading.
        let mut completed = false;
        for _ in 0..30 {
            let props = client.get_torrent_properties(&hash).await.unwrap();
            if props.completion_date.is_some_and(|date| date >= 0) {
                completed = true;
                break;
            }
            sleep(std::time::Duration::from_secs(1)).await;
        }
        assert!(completed, "alice torrent did not complete in time");

        let data = client
            .download_torrent_file(&hash, "0")
            .await
            .unwrap();
        let content = String::from_utf8(data.to_vec()).unwrap();
        assert_eq!(content, expected);
    }
}

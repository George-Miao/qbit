#![doc = include_str!("../README.md")]
#![warn(clippy::future_not_send)]
#![cfg_attr(test, feature(lazy_cell))]

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::Debug,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
};

use http_client::{
    http_types::{headers, Method, StatusCode, Url},
    Body, HttpClient, Request, Response,
};
pub mod model;
use serde::Serialize;
use serde_with::skip_serializing_none;
use tap::{Pipe, TapFallible};
use tracing::{debug, trace, warn};

use crate::{ext::*, model::*};

mod ext;

/// Main entry point of the library. It provides a high-level API to interact
/// with qBittorrent WebUI API.
pub struct Qbit<C> {
    client: C,
    endpoint: Url,
    credential: Credential,
    cookie: Mutex<Option<String>>,
}

impl<C: HttpClient> Qbit<C> {
    pub fn new<U>(endpoint: U, credential: Credential, client: C) -> Self
    where
        U: TryInto<Url>,
        U::Error: Debug,
    {
        Self {
            client,
            endpoint: endpoint.try_into().expect("Invalid endpoint URL"),
            credential,
            cookie: Mutex::new(None),
        }
    }

    pub fn new_with_cookie<U>(endpoint: U, cookie: String, client: C) -> Self
    where
        U: TryInto<Url>,
        U::Error: Debug,
    {
        Self {
            client,
            endpoint: endpoint.try_into().expect("Invalid endpoint URL"),
            credential: Credential::dummy(),
            cookie: Mutex::from(Some(cookie)),
        }
    }

    pub async fn get_cookie(&self) -> Result<Option<String>> {
        Ok(self.cookie.lock().unwrap().deref().clone())
    }

    pub async fn logout(&self) -> Result<()> {
        self.get("auth/logout", NONE).await?.end()
    }

    pub async fn get_version(&self) -> Result<String> {
        self.get("app/version", NONE)
            .await?
            .body_string()
            .await
            .map_err(Into::into)
    }

    pub async fn get_webapi_version(&self) -> Result<String> {
        self.get("app/webapiVersion", NONE)
            .await?
            .body_string()
            .await
            .map_err(Into::into)
    }

    pub async fn get_build_info(&self) -> Result<BuildInfo> {
        self.get("app/buildInfo", NONE)
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.get("app/shutdown", NONE).await?.end()
    }

    pub async fn get_preferences(&self) -> Result<Preferences> {
        self.get("app/preferences", NONE)
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn set_preferences(
        &self,
        preferences: impl Borrow<Preferences> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            json: &'a Preferences,
        }

        self.post(
            "app/setPreferences",
            NONE,
            Some(&Arg {
                json: preferences.borrow(),
            }),
        )
        .await?
        .end()
    }

    pub async fn get_default_save_path(&self) -> Result<PathBuf> {
        self.get("app/defaultSavePath", NONE)
            .await?
            .body_string()
            .await
            .map_err(Into::into)
            .map(PathBuf::from)
    }

    pub async fn get_logs(&self, arg: impl Borrow<GetLogsArg> + Send + Sync) -> Result<Vec<Log>> {
        self.get("log/main", Some(arg.borrow()))
            .await?
            .body_json()
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

        self.get(
            "log/peers",
            Some(&Arg {
                last_known_id: last_known_id.into(),
            }),
        )
        .await?
        .body_json()
        .await
        .map_err(Into::into)
    }

    pub async fn sync(&self, rid: impl Into<Option<i64>> + Send + Sync) -> Result<SyncData> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            rid: Option<i64>,
        }

        self.get("sync/maindata", Some(&Arg { rid: rid.into() }))
            .await?
            .body_json()
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

        self.get(
            "sync/torrentPeers",
            Some(&Arg {
                hash: hash.as_ref(),
                rid: rid.into(),
            }),
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .body_json()
        .await
        .map_err(Into::into)
    }

    pub async fn get_transfer_info(&self) -> Result<TransferInfo> {
        self.get("transfer/info", NONE)
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_speed_limits_mode(&self) -> Result<bool> {
        self.get("transfer/speedLimitsMode", NONE)
            .await?
            .body_string()
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
        self.get("transfer/toggleSpeedLimitsMode", NONE)
            .await?
            .end()
    }

    pub async fn get_download_limit(&self) -> Result<u64> {
        self.get("transfer/downloadLimit", NONE)
            .await?
            .body_string()
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

        self.get("transfer/setDownloadLimit", Some(&Arg { limit }))
            .await?
            .end()
    }

    pub async fn get_upload_limit(&self) -> Result<u64> {
        self.get("transfer/uploadLimit", NONE)
            .await?
            .body_string()
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

        self.get("transfer/setUploadLimit", Some(&Arg { limit }))
            .await?
            .end()
    }

    pub async fn ban_peers(&self, peers: impl Into<Sep<String, '|'>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            peers: String,
        }

        self.get(
            "transfer/banPeers",
            Some(&Arg {
                peers: peers.into().to_string(),
            }),
        )
        .await?
        .end()
    }

    pub async fn get_torrent_list(&self, arg: GetTorrentListArg) -> Result<Vec<Torrent>> {
        self.get("torrents/info", Some(&arg))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_properties(
        &self,
        hash: impl AsRef<str> + Sync + Send + Sync,
    ) -> Result<TorrentProperty> {
        self.get("torrents/properties", Some(&HashArg::new(hash.as_ref())))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_trackers(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<Tracker>> {
        self.get("torrents/trackers", Some(&HashArg::new(hash.as_ref())))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_web_seeds(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<WebSeed>> {
        self.get("torrents/webseeds", Some(&HashArg::new(hash.as_ref())))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .body_json()
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

        self.get(
            "torrents/files",
            Some(&Arg {
                hash: hash.as_ref(),
                indexes: indexes.into().map(|s| s.to_string()),
            }),
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .body_json()
        .await
        .map_err(Into::into)
    }

    pub async fn get_torrent_pieces_states(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<PieceState>> {
        self.get("torrents/pieceStates", Some(&HashArg::new(hash.as_ref())))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn get_torrent_pieces_hashes(
        &self,
        hash: impl AsRef<str> + Send + Sync,
    ) -> Result<Vec<String>> {
        self.get("torrents/pieceHashes", Some(&HashArg::new(hash.as_ref())))
            .await
            .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn pause_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/pause", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn resume_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/resume", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn delete_torrents(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
        delete_files: impl Into<Option<bool>> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            hashes: Hashes,
            delete_files: Option<bool>,
        }
        self.get(
            "torrents/delete",
            Some(&Arg {
                hashes: hashes.into(),
                delete_files: delete_files.into(),
            }),
        )
        .await?
        .body_json()
        .await
        .map_err(Into::into)
    }

    pub async fn recheck_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/recheck", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn reannounce_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/reannounce", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn add_torrent(
        &self,
        arg: impl Borrow<AddTorrentArg> + Send + Sync,
    ) -> Result<Vec<Torrent>> {
        self.post("torrents/add", NONE, Some(arg.borrow()))
            .await?
            .body_json()
            .await
            .map_err(Into::into)
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

        self.get(
            "torrents/addTrackers",
            Some(&Arg {
                hash: hash.as_ref(),
                urls: urls.into().to_string(),
            }),
        )
        .await
        .and_then(|r| r.map_status(TORRENT_NOT_FOUND))?
        .body_json()
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
        struct EditTrackerArg<'a> {
            hash: &'a str,
            orig_url: Url,
            new_url: Url,
        }
        self.get(
            "torrents/editTracker",
            Some(&EditTrackerArg {
                hash: hash.as_ref(),
                orig_url,
                new_url,
            }),
        )
        .await?
        .map_status(|c| {
            use StatusCode::*;
            match c {
                BadRequest => Some(Error::ApiError(ApiError::InvalidTrackerUrl)),
                NotFound => Some(Error::ApiError(ApiError::TorrentNotFound)),
                Conflict => Some(Error::ApiError(ApiError::ConflictTrackerUrl)),
                _ => None,
            }
        })?
        .body_json()
        .await
        .map_err(Into::into)
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

        self.get(
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

        self.get(
            "torrents/addPeers",
            Some(&AddPeersArg {
                hash: hashes.into().to_string(),
                peers: peers.into(),
            }),
        )
        .await
        .and_then(|r| {
            r.map_status(|c| {
                if c == StatusCode::BadRequest {
                    Some(Error::ApiError(ApiError::InvalidPeers))
                } else {
                    None
                }
            })
        })?
        .end()
    }

    pub async fn increase_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/increasePrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::Conflict {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn decrease_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/decreasePrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::Conflict {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn maximal_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/topPrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::Conflict {
                    Some(Error::ApiError(ApiError::QueueingDisabled))
                } else {
                    None
                }
            })?;
        Ok(())
    }

    pub async fn minimal_priority(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.get("torrents/bottomPrio", Some(&HashesArg::new(hashes)))
            .await?
            .map_status(|c| {
                if c == StatusCode::Conflict {
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

        self.get(
            "torrents/filePrio",
            Some(&SetFilePriorityArg {
                hash: hash.as_ref(),
                id: indexes.into(),
                priority,
            }),
        )
        .await?
        .map_status(|c| {
            use StatusCode::*;

            match c {
                BadRequest => panic!("Invalid priority or id. This is a bug."),
                NotFound => Some(Error::ApiError(ApiError::TorrentNotFound)),
                Conflict => Some(Error::ApiError(ApiError::MetaNotDownloadedOrIdNotFound)),
                _ => None,
            }
        })?;
        Ok(())
    }

    pub async fn get_torrent_download_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<HashMap<String, u64>> {
        self.get("torrents/downloadLimit", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
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

        self.get(
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
        self.get("torrents/setShareLimits", Some(arg.borrow()))
            .await?
            .end()
    }

    pub async fn get_torrent_upload_limit(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<HashMap<String, u64>> {
        self.get("torrents/uploadLimit", Some(&HashesArg::new(hashes)))
            .await?
            .body_json()
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

        self.get(
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

        self.get(
            "torrents/setLocation",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                location: location.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            use StatusCode::*;

            match c {
                BadRequest => Some(Error::ApiError(ApiError::SavePathEmpty)),
                Forbidden => Some(Error::ApiError(ApiError::NoWriteAccess)),
                Conflict => Some(Error::ApiError(ApiError::UnableToCreateDir)),
                _ => None,
            }
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

        self.get(
            "torrents/rename",
            Some(&RenameArg {
                hash: hash.as_ref(),
                name: name.as_str(),
            }),
        )
        .await?
        .map_status(|c| {
            use StatusCode::*;

            match c {
                NotFound => Some(Error::ApiError(ApiError::TorrentNotFound)),
                Conflict => panic!("Name should not be empty. This is a bug."),
                _ => None,
            }
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

        self.get(
            "torrents/setCategory",
            Some(&Arg {
                hashes: hashes.into().to_string(),
                category: category.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::Conflict {
                Some(Error::ApiError(ApiError::CategoryNotFound))
            } else {
                None
            }
        })?
        .end()
    }

    pub async fn get_categories(&self) -> Result<HashMap<String, Category>> {
        self.get("torrents/categories", NONE)
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn add_category<T: AsRef<str> + Send + Sync>(
        &self,
        category: NonEmptyStr<T>,
        save_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Arg<'a> {
            category: &'a str,
            save_path: &'a Path,
        }

        self.get(
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
        struct Arg<'a> {
            category: &'a str,
            save_path: &'a Path,
        }

        self.get(
            "torrents/createCategory",
            Some(&Arg {
                category: category.as_str(),
                save_path: save_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::Conflict {
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

        self.get(
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

        self.get(
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

        self.get(
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
        self.get("torrents/tags", NONE)
            .await?
            .body_json()
            .await
            .map_err(Into::into)
    }

    pub async fn create_tags(&self, tags: impl Into<Sep<String, ','>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            tags: String,
        }

        self.get(
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

        self.get(
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

        self.get(
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
        self.get(
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
        self.get(
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

        self.get(
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

        self.get(
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
        struct Arg<'a> {
            hash: &'a str,
            old_path: &'a Path,
            new_path: &'a Path,
        }

        self.get(
            "torrents/renameFile",
            Some(&Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::Conflict {
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
        struct Arg<'a> {
            hash: &'a str,
            old_path: &'a Path,
            new_path: &'a Path,
        }

        self.get(
            "torrents/renameFolder",
            Some(&Arg {
                hash: hash.as_ref(),
                old_path: old_path.as_ref(),
                new_path: new_path.as_ref(),
            }),
        )
        .await?
        .map_status(|c| {
            if c == StatusCode::Conflict {
                Error::ApiError(ApiError::InvalidPath).pipe(Some)
            } else {
                None
            }
        })?
        .end()
    }

    fn url(&self, path: &'static str) -> Url {
        self.endpoint
            .join("api/v2/")
            .unwrap()
            .join(path)
            .expect("Invalid API endpoint")
    }

    /// Log in to qBittorrent. Set force to `true` to forcefully re-login
    /// regardless if cookie is already set.
    pub async fn login(&self, force: bool) -> Result<()> {
        let re_login = force || { self.cookie.lock().unwrap().is_none() };
        if re_login {
            debug!("Cookie not found, logging in");
            let mut req = Request::get(self.url("auth/login"));
            req.set_query(&self.credential)?;
            let Cookie(cookie) = self
                .client
                .send(req)
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::Forbidden => Some(Error::ApiError(ApiError::IpBanned)),
                    _ => None,
                })?
                .extract::<Cookie>()?;

            // Ignore result
            drop(self.cookie.lock().unwrap().replace(cookie));

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
        qs: Option<&(impl Serialize + Sync)>,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        for i in 0..3 {
            // If it's not the first attempt, we need to re-login
            self.login(i != 0).await?;

            let mut req = Request::new(method, self.url(path));

            req.append_header(headers::COOKIE, {
                self.cookie
                    .lock()
                    .unwrap()
                    .as_deref()
                    .expect("Cookie should be set after login")
            });

            if let Some(qs) = qs {
                req.set_query(qs)?;
            }

            if let Some(ref body) = body {
                req.set_body(Body::from_form(body)?);
            }

            trace!(request = ?req, "Sending request");
            let res = self
                .client
                .send(req)
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::Forbidden => Some(Error::ApiError(ApiError::NotLoggedIn)),
                    _ => None,
                })
                .tap_ok(|res| trace!(?res));
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

    // pub async fn add_torrent(&self, urls: )
    async fn get(
        &self,
        path: &'static str,
        qs: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        self.request(Method::Get, path, qs, NONE).await
    }

    async fn post(
        &self,
        path: &'static str,
        qs: Option<&(impl Serialize + Sync)>,
        body: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        self.request(Method::Post, path, qs, body).await
    }
}

const NONE: Option<&'static ()> = Option::None;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http error: {0}")]
    HttpError(http_client::Error),

    #[error("API Returned bad response: {explain}")]
    BadResponse { explain: &'static str },

    #[error("API returned unknown status code: {0}")]
    UnknownHttpCode(StatusCode),

    #[error(transparent)]
    ApiError(#[from] ApiError),
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

    #[error("Torrent metadata hasn't downloaded yet or At least one file id was not found")]
    MetaNotDownloadedOrIdNotFound,

    #[error("Save path is empty")]
    SavePathEmpty,

    #[error("User does not have write access to directory")]
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

impl From<http_client::Error> for Error {
    fn from(err: http_client::Error) -> Self {
        Self::HttpError(err)
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod test {
    use std::{
        env,
        sync::{LazyLock, OnceLock},
    };

    use http_client::h1::H1Client;
    use tracing::info;

    use super::*;

    async fn prepare<'a>() -> Result<&'a Qbit<H1Client>> {
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
        static API: OnceLock<Qbit<H1Client>> = OnceLock::new();

        if let Some(api) = API.get() {
            Ok(api)
        } else {
            let (credential, url) = PREPARE.deref().clone();
            let api = Qbit::new(url, credential, H1Client::new());
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
    async fn test_a() {
        let client = prepare().await.unwrap();

        client
            .set_preferences(&Preferences::default())
            .await
            .unwrap();
    }
}

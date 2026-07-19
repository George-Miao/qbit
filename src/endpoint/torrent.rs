use std::{borrow::Borrow, collections::HashMap, path::Path};

use bytes::Bytes;
use serde::Serialize;
use serde_with::skip_serializing_none;
use tap::Pipe;

#[cfg(feature = "cyper")]
use crate::client::PartExt;
use crate::{
    ApiError, Error, Qbit, Result,
    client::{CheckError, Method, RequestBuilder, StatusCode, Url, multipart},
    ext::*,
    model::*,
};

impl Qbit {
    /// Return torrents matching the supplied filters and pagination options.
    pub async fn get_torrent_list(&self, arg: GetTorrentListArg) -> Result<Vec<Torrent>> {
        self.get_with("torrents/info", &arg)
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Export the torrent metadata file for the supplied hash.
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

    /// Return generic properties for the supplied torrent.
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

    /// Return trackers for the supplied torrent.
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

    /// Return web seeds for the supplied torrent.
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

    /// Return file content information for the supplied torrent.
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

    /// Return the download state of each piece in the supplied torrent.
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

    /// Return the hash of each piece in the supplied torrent.
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

    /// Stop the supplied torrents.
    pub async fn stop_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/stop", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Start the supplied torrents.
    pub async fn start_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/start", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Delete the supplied torrents, optionally including their downloaded
    /// files.
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

    /// Recheck the supplied torrents against their downloaded data.
    pub async fn recheck_torrents(&self, hashes: impl Into<Hashes> + Send + Sync) -> Result<()> {
        self.post_with("torrents/recheck", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Reannounce the supplied torrents to their trackers.
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

    /// Add one or more torrents from URLs or torrent files.
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
            TorrentSource::Urls { urls: _ } => self.post_with("torrents/add", args).await?.end(),
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
                    StatusCode::CONFLICT => Some(Error::ApiError(ApiError::TorrentAddFailed)),
                    _ => None,
                })?
                .end(),
        }
    }

    /// Add trackers to the supplied torrent.
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

    /// Replace a tracker URL on the supplied torrent.
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

    /// Remove trackers from the supplied torrent.
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

    /// Add peers to the supplied torrents.
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

    /// Increase the queue priority of the supplied torrents.
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

    /// Decrease the queue priority of the supplied torrents.
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

    /// Move the supplied torrents to the top of the queue.
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

    /// Move the supplied torrents to the bottom of the queue.
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

    /// Set the download priority of selected files in a torrent.
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

    /// Return the download limit for the supplied torrent.
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

    /// Set the download limit for the supplied torrents.
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

    /// Set share-ratio and seeding-time limits for the supplied torrents.
    pub async fn set_torrent_shared_limit(
        &self,
        arg: impl Borrow<SetTorrentSharedLimitArg> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/setShareLimits", arg.borrow())
            .await?
            .end()
    }

    /// Return the upload limit for the supplied torrent.
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

    /// Set the upload limit for the supplied torrents.
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

    /// Set the save location for the supplied torrents.
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

    /// Rename the supplied torrent.
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

    /// Set the category of the supplied torrents.
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

    /// Return all torrent categories.
    pub async fn get_categories(&self) -> Result<HashMap<String, Category>> {
        self.get("torrents/categories")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Create a torrent category with the supplied save path.
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

    /// Update the save path of a torrent category.
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

    /// Remove the supplied torrent categories.
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

    /// Add tags to the supplied torrents.
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

    /// Remove tags from the supplied torrents.
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

    /// Return all torrent tags.
    pub async fn get_all_tags(&self) -> Result<Vec<String>> {
        self.get("torrents/tags")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Create the supplied torrent tags.
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

    /// Delete the supplied torrent tags.
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

    /// Enable or disable automatic torrent management for the supplied
    /// torrents.
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

    /// Toggle sequential downloading for the supplied torrents.
    pub async fn toggle_sequential_download(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/toggleSequentialDownload", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Toggle first and last piece priority for the supplied torrents.
    pub async fn toggle_first_last_piece_priority(
        &self,
        hashes: impl Into<Hashes> + Send + Sync,
    ) -> Result<()> {
        self.post_with("torrents/toggleFirstLastPiecePrio", &HashesArg::new(hashes))
            .await?
            .end()
    }

    /// Enable or disable force start for the supplied torrents.
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

    /// Enable or disable super seeding for the supplied torrents.
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

    /// Rename a file within the supplied torrent.
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

    /// Rename a folder within the supplied torrent.
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
}

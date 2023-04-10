#![warn(clippy::future_not_send)]

use std::{path::PathBuf, sync::OnceLock};

use http_client::{
    http_types::{cookies::Cookie as HttpCookie, headers, StatusCode, Url},
    Body, HttpClient, Request, Response,
};
use serde::Serialize;
use tap::Pipe;

use crate::model::{
    BuildInfo, Cookie, Hashes, Log, PeerLog, PieceState, Preferences, ResponseExt, Torrent,
    TorrentContent, TorrentFilter, TorrentProperty, Tracker, TransferInfo, WebSeed,
};

mod model;

pub struct Api<C> {
    client: C,
    url: Url,
    cookie: OnceLock<HttpCookie<'static>>,
}

impl<C: HttpClient> Api<C> {
    pub fn new(url: Url, client: C) -> Self {
        Self {
            client,
            url,
            cookie: OnceLock::new(),
        }
    }

    pub async fn login(
        &self,
        username: impl AsRef<str> + Send,
        password: impl AsRef<str> + Send,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct Login<'a> {
            username: &'a str,
            password: &'a str,
        }

        let (username, password) = (username.as_ref(), password.as_ref());

        if self.cookie.get().is_none() {
            let cookie = self
                .get("login", Login { username, password })
                .await?
                .handle_status(|code| match code as _ {
                    StatusCode::Forbidden => Some(Error::ApiError(ApiError::IpBanned)),
                    _ => None,
                })?
                .extract::<Cookie>()?
                .cookie;

            // Ignore result
            drop(self.cookie.set(cookie));
        }

        Ok(())
    }

    pub async fn logout(&self) -> Result<()> {
        todo!()
    }

    pub async fn get_version(&self) -> Result<String> {
        todo!()
    }

    pub async fn get_webapi_version(&self) -> Result<String> {
        todo!()
    }

    pub async fn get_build_info(&self) -> Result<BuildInfo> {
        todo!()
    }

    pub async fn get_preferences(&self) -> Result<Preferences> {
        todo!()
    }

    pub async fn set_preferences(&self, preferences: Preferences) -> Result<Preferences> {
        todo!()
    }

    pub async fn get_default_save_path(&self) -> Result<PathBuf> {
        todo!()
    }

    pub async fn get_logs(&self) -> Result<Vec<Log>> {
        todo!()
    }

    pub async fn get_peer_logs(
        &self,
        last_known_id: impl Into<Option<i64>> + Send,
    ) -> Result<Vec<PeerLog>> {
        todo!()
    }

    pub async fn sync(&self, rid: impl Into<Option<i64>> + Send) -> Result<Vec<PeerLog>> {
        todo!()
    }

    // pub async fn torrent_peers(&self, hash: impl AsRef<&str> + Send, rid: impl
    // Into<Option<i64>> + Send) -> Result<Vec<PeerLog>> { todo!() }
    pub async fn get_transfer_info(&self) -> Result<TransferInfo> {
        todo!()
    }

    pub async fn get_speed_limits_mode(&self) -> Result<bool> {
        todo!()
    }

    pub async fn toggle_speed_limits_mode(&self) -> Result<bool> {
        todo!()
    }

    pub async fn get_download_limit(&self) -> Result<Option<u64>> {
        todo!()
    }

    pub async fn set_download_limit(&self, limit: u64) -> Result<()> {
        todo!()
    }

    pub async fn get_upload_limit(&self) -> Result<Option<u64>> {
        todo!()
    }

    pub async fn set_upload_limit(&self, limit: u64) -> Result<()> {
        todo!()
    }

    pub async fn ban_peers(&self, peers: impl AsRef<[String]> + Send) -> Result<()> {
        todo!()
    }

    pub async fn get_torrent_list(
        &self,
        filter: impl Into<Option<TorrentFilter>> + Send,
        category: impl Into<Option<String>> + Send,
        tag: impl Into<Option<String>> + Send,
        sort: impl Into<Option<String>> + Send,
        reverse: impl Into<Option<bool>> + Send,
        limit: impl Into<Option<u64>> + Send,
        offset: impl Into<Option<u64>> + Send,
        hashes: impl Into<Option<Vec<String>>> + Send,
    ) -> Result<Vec<Torrent>> {
        todo!()
    }

    pub async fn get_torrent_properties(
        &self,
        hash: impl AsRef<str> + Send,
    ) -> Result<TorrentProperty> {
        todo!()
    }

    pub async fn get_torrent_trackers(&self, hash: impl AsRef<str> + Send) -> Result<Vec<Tracker>> {
        todo!()
    }

    pub async fn get_torrent_web_seeds(
        &self,
        hash: impl AsRef<str> + Send,
    ) -> Result<Vec<WebSeed>> {
        todo!()
    }

    pub async fn get_torrent_contents(
        &self,
        hash: impl AsRef<str> + Send,
        indexes: impl Into<Vec<String>>,
    ) -> Result<Vec<TorrentContent>> {
        todo!()
    }

    pub async fn get_torrent_pieces_states(
        &self,
        hash: impl AsRef<str> + Send,
    ) -> Result<Vec<PieceState>> {
        todo!()
    }

    pub async fn get_torrent_pieces_hashes(
        &self,
        hash: impl AsRef<str> + Send,
    ) -> Result<Vec<String>> {
        todo!()
    }

    pub async fn pause_torrents(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn resume_torrents(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn delete_torrents(
        &self,
        hashes: impl Into<Hashes> + Send,
        delete_files: impl Into<Option<bool>> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn recheck_torrents(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn reannounce_torrents(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    // pub async fn add_torrent(&self, urls: )
    async fn get(&self, path: &'static str, qs: impl Serialize + Send + Sync) -> Result<Response> {
        self.url
            .join(path)
            .expect("Invalid API path")
            .pipe(Request::get)
            .pipe(|mut r| {
                if let Some(cookie) = self.cookie.get() {
                    r.append_header(headers::COOKIE, cookie.to_string())
                }
                r.set_query(&qs)?;
                Result::<_>::Ok(r)
            })?
            .pipe(|req| self.client.send(req))
            .await
            .map_err(Into::into)
    }

    async fn post(
        &self,
        path: &'static str,
        qs: impl Serialize + Send + Sync,
        body: impl Serialize + Send + Sync,
    ) -> Result<Response> {
        self.url
            .join(path)
            .expect("Invalid API path")
            .pipe(Request::post)
            .pipe(|mut r| {
                if let Some(cookie) = self.cookie.get() {
                    r.append_header(headers::COOKIE, cookie.to_string())
                }

                r.set_query(&qs)?;
                r.set_body(Body::from_json(&body)?);
                Result::<_, Error>::Ok(r)
            })?
            .pipe(|req| self.client.send(req))
            .await
            .map_err(Into::into)
    }
}

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

/// Errors defined and returned by the API with status code
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User's IP is banned for too many failed login attempts")]
    IpBanned,

    #[error("API routes requires authentication")]
    Unauthorized,

    #[error("Torrent hash not found: {0}")]
    TorrentHashNotFound(String),
}

impl From<http_client::Error> for Error {
    fn from(err: http_client::Error) -> Self {
        Self::HttpError(err)
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#![warn(clippy::future_not_send)]
#![cfg_attr(test, feature(lazy_cell))]

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use http_client::{
    http_types::{headers, Method, StatusCode, Url},
    Body, HttpClient, Request, Response,
};
use serde::Serialize;
use tap::TapFallible;
use tracing::{debug, trace};

use crate::{
    ext::*,
    model::{
        AddTorrentArg, BuildInfo, Category, Credential, GetTorrentListArg, Hashes, Log, PeerLog,
        PieceState, Preferences, Priority, Sep, SetTorrentSharedLimitArg, Torrent, TorrentContent,
        TorrentProperty, TorrentSource, Tracker, TransferInfo, WebSeed,
    },
};

mod ext;
mod model;

pub struct Api<C> {
    client: C,
    endpoint: Url,
    credential: Credential,
    cookie: OnceLock<String>,
}

impl<C: HttpClient> Api<C> {
    pub fn new(endpoint: Url, credential: Credential, client: C) -> Self {
        Self {
            client,
            endpoint,
            credential,
            cookie: OnceLock::new(),
        }
    }

    pub fn new_with_cookie(endpoint: Url, cookie: String, client: C) -> Self {
        Self {
            client,
            endpoint,
            credential: Credential {
                username: String::new(),
                password: String::new(),
            },
            cookie: OnceLock::from(cookie),
        }
    }

    pub async fn get_cookie(&self) -> Result<Option<String>> {
        Ok(self.cookie.get().cloned())
    }

    pub async fn logout(&self) -> Result<()> {
        self.get("auth/logout", NONE).await.map(|_| ())
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

    pub async fn ban_peers(&self, peers: impl Into<Sep<String, '|'>> + Send) -> Result<()> {
        todo!()
    }

    pub async fn get_torrent_list(&self, arg: GetTorrentListArg) -> Result<Vec<Torrent>> {
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
        indexes: impl Into<Sep<String, '|'>> + Send,
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

    pub async fn add_torrent(
        &self,
        src: TorrentSource,
        arg: AddTorrentArg,
    ) -> Result<Vec<Torrent>> {
        todo!()
    }

    pub async fn add_trackers(
        &self,
        hash: impl AsRef<str> + Send,
        urls: impl Into<Sep<String, '\n'>> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn edit_trackers(
        &self,
        hash: impl AsRef<str> + Send,
        orig_url: Url,
        new_url: Url,
    ) -> Result<()> {
        todo!()
    }

    pub async fn remove_trackers(
        &self,
        hash: impl AsRef<str> + Send,
        url: impl AsRef<str> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn add_peers(
        &self,
        hash: impl AsRef<str> + Send,
        peers: impl Into<Sep<String, '|'>> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn increase_priority(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn decrease_priority(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn maximal_priority(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn minimal_priority(&self, hashes: impl Into<Hashes> + Send) -> Result<()> {
        todo!()
    }

    pub async fn set_file_priority(
        &self,
        hash: impl AsRef<str> + Send,
        indexes: impl Into<Sep<i64, '|'>> + Send,
        priority: Priority,
    ) -> Result<()> {
        todo!()
    }

    pub async fn get_torrent_download_limit(
        &self,
        hashes: impl Into<Hashes> + Send,
    ) -> Result<HashMap<String, u64>> {
        todo!()
    }

    pub async fn set_torrent_download_limit(
        &self,
        hashes: impl Into<Hashes> + Send,
        limit: u64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_torrent_shared_limit(&self, arg: SetTorrentSharedLimitArg) -> Result<()> {
        todo!()
    }

    pub async fn get_torrent_upload_limit(
        &self,
        hashes: impl Into<Hashes> + Send,
    ) -> Result<HashMap<String, u64>> {
        todo!()
    }

    pub async fn set_torrent_upload_limit(
        &self,
        hashes: impl Into<Hashes> + Send,
        limit: u64,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_torrent_location(
        &self,
        hashes: impl Into<Hashes> + Send,
        location: impl AsRef<str> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_torrent_name(
        &self,
        hash: impl AsRef<str> + Send,
        name: impl AsRef<str> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_torrent_category(
        &self,
        hashes: impl Into<Hashes> + Send,
        category: impl AsRef<str> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn get_categories(&self) -> Result<HashMap<String, Category>> {
        todo!()
    }

    pub async fn add_category(
        &self,
        category: impl AsRef<str> + Send,
        save_path: impl AsRef<Path> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn edit_category(
        &self,
        category: impl AsRef<str> + Send,
        save_path: impl AsRef<Path> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn remove_categories(
        &self,
        categories: impl Into<Sep<String, '\n'>> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn add_torrent_tags(
        &self,
        hashes: impl Into<Hashes> + Send,
        tags: impl Into<Sep<String, '\n'>> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn remove_torrent_tags(
        &self,
        hashes: impl Into<Hashes> + Send,
        tags: Option<impl Into<Sep<String, '\n'>> + Send>,
    ) -> Result<()> {
        todo!()
    }

    pub async fn get_all_tags(&self) -> Result<Vec<String>> {
        todo!()
    }

    pub async fn create_tags(&self, tags: impl Into<Sep<String, ','>> + Send) -> Result<()> {
        todo!()
    }

    pub async fn delete_tags(&self, tags: impl Into<Sep<String, ','>> + Send) -> Result<()> {
        todo!()
    }

    pub async fn set_auto_management(
        &self,
        hashes: impl Into<Hashes> + Send,
        enable: bool,
    ) -> Result<()> {
        todo!()
    }

    pub async fn toggle_torrent_sequential_download(
        &self,
        hashes: impl Into<Hashes> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn toggle_first_last_piece_priority(
        &self,
        hashes: impl Into<Hashes> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_force_start(
        &self,
        hashes: impl Into<Hashes> + Send,
        value: bool,
    ) -> Result<()> {
        todo!()
    }

    pub async fn set_super_seeding(
        &self,
        hashes: impl Into<Hashes> + Send,
        value: bool,
    ) -> Result<()> {
        todo!()
    }

    pub async fn rename_file(
        &self,
        hash: impl AsRef<str> + Send,
        old_path: impl AsRef<Path> + Send,
        new_path: impl AsRef<Path> + Send,
    ) -> Result<()> {
        todo!()
    }

    pub async fn rename_folder(
        &self,
        hash: impl AsRef<str> + Send,
        old_path: impl AsRef<Path> + Send,
        new_path: impl AsRef<Path> + Send,
    ) -> Result<()> {
        todo!()
    }

    fn url(&self, path: &'static str) -> Url {
        self.endpoint
            .join("api/v2/")
            .unwrap()
            .join(path)
            .expect("Invalid API endpoint")
    }

    async fn login(&self) -> Result<()> {
        if self.cookie.get().is_none() {
            debug!("Cookie not found, logging in");
            let mut req = Request::get(self.url("auth/login"));
            req.set_query(&self.credential)?;
            let Cookie(cookie) = self
                .client
                .send(req)
                .await?
                .handle_status(|code| match code as _ {
                    StatusCode::Forbidden => Some(Error::ApiError(ApiError::IpBanned)),
                    _ => None,
                })?
                .extract::<Cookie>()?;

            // Ignore result
            drop(self.cookie.set(cookie));

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
        self.login().await?;
        let mut req = Request::new(method, self.url(path));

        req.append_header(
            headers::COOKIE,
            self.cookie.get().expect("Cookie should be set after login"),
        );

        if let Some(qs) = qs {
            req.set_query(qs)?;
        }

        if let Some(body) = body {
            req.set_body(Body::from_json(body)?);
        }

        trace!(request = ?req, "Sending request");

        self.client
            .send(req)
            .await
            .map_err(Into::into)
            .tap_ok(|res| trace!(?res))
    }

    // pub async fn add_torrent(&self, urls: )
    async fn get(
        &self,
        path: &'static str,
        qs: Option<&(impl Serialize + Sync)>,
    ) -> Result<Response> {
        self.request(Method::Get, path, qs, Option::<&()>::None)
            .await
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

#[cfg(test)]
mod test {
    use std::{env, sync::LazyLock};

    use http_client::h1::H1Client;
    use tracing::info;

    use super::*;

    async fn prepare<'a>() -> Result<&'a Api<H1Client>> {
        static PREPARE: LazyLock<(Credential, Url)> = LazyLock::new(|| {
            dotenv::dotenv().expect("Failed to load .env file");
            tracing_subscriber::fmt::init();

            (
                Credential {
                    username: env::var("QBIT_USERNAME").expect("QBIT_USERNAME not set"),
                    password: env::var("QBIT_PASSWORD").expect("QBIT_PASSWORD not set"),
                },
                env::var("QBIT_BASEURL")
                    .expect("QBIT_BASEURL not set")
                    .parse()
                    .expect("QBIT_BASEURL is not a valid url"),
            )
        });
        static API: OnceLock<Api<H1Client>> = OnceLock::new();

        if let Some(api) = API.get() {
            Ok(api)
        } else {
            let (credential, url) = &*PREPARE;
            let api = Api::new(url.to_owned(), credential.clone(), H1Client::new());
            api.login().await?;
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
}

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]

use std::{
    fmt::Debug,
    sync::{Mutex, MutexGuard},
};

mod client;
use client::*;

pub mod model;
pub use builder::QbitBuilder;
use serde::Serialize;
use tap::TapFallible;
use tracing::{trace, warn};
use url::Url;

use crate::{ext::*, model::Credential};

mod builder;
mod endpoint;
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
                let header_value =
                    header_value.expect("Should always have header value if logged in");
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

    /// Search could not start because Python is unavailable or the concurrent
    /// search limit was reached
    #[error("Search is unavailable")]
    SearchUnavailable,

    /// Search job does not exist
    #[error("Search job not found")]
    SearchJobNotFound,

    /// Search result offset is outside the available result range
    #[error("Search result offset is out of range")]
    SearchInvalidOffset,
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod test;

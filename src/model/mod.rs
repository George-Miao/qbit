use std::path::PathBuf;

use http_client::{http_types::StatusCode, Response};
use serde::{
    de::{Deserializer, Error as DesError},
    Deserialize, Serialize,
};

use crate::{ApiError, Error};

mod_use::mod_use![cookie, app, log, sync, torrent, transfer, search];

pub trait FromResponse {
    fn from_response(response: &Response) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ResponseExt: Sized {
    fn extract<T: FromResponse>(&self) -> Result<T, Error>;

    fn handle_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self, Error>;
}

impl ResponseExt for Response {
    fn extract<T: FromResponse>(&self) -> Result<T, Error> {
        T::from_response(self)
    }

    fn handle_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self, Error> {
        let status = self.status();

        if status.is_success() {
            Ok(self)
        } else {
            match f(status) {
                Some(err) => Err(err),
                None => match status {
                    StatusCode::Forbidden => Err(Error::ApiError(ApiError::Unauthorized)),
                    code => Err(Error::UnknownHttpCode(code)),
                },
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub(crate) struct Empty;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Category {
    pub name: String,
    pub save_path: PathBuf,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Tracker {
    /// Tracker url
    pub url: String,
    /// Tracker status. See the table below for possible values
    pub status: TrackerStatus,
    /// Tracker priority tier. Lower tier trackers are tried before higher
    /// tiers. Tier numbers are valid when `>= 0`, `< 0` is used as placeholder
    /// when `tier` does not exist for special entries (such as DHT).
    pub tier: i64,
    /// Number of peers for current torrent, as reported by the tracker
    pub num_peers: i64,
    /// Number of seeds for current torrent, asreported by the tracker
    pub num_seeds: i64,
    /// Number of leeches for current torrent, as reported by the tracker
    pub num_leeches: i64,
    /// Number of completed downlods for current torrent, as reported by the
    /// tracker
    pub num_downloaded: i64,
    /// Tracker message (there is no way of knowing what this message is - it's
    /// up to tracker admins)
    pub msg: String,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(i8)]
pub enum TrackerStatus {
    /// Tracker is disabled (used for DHT, PeX, and LSD)
    Disabled     = 0,
    /// Tracker has not been contacted yet
    NotContacted = 1,
    /// Tracker has been contacted and is working
    Working      = 2,
    /// Tracker is updating
    Updating     = 3,
    /// Tracker has been contacted, but it is not working (or doesn't send
    /// proper replies)
    NotWorking   = 4,
}

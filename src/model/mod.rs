use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use tap::Pipe;

mod_use::mod_use![app, log, sync, torrent, transfer, search];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
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

/// A wrapper around `Vec<T>` that implements `FromStr` and `ToString` as
/// `C`-separated strings where `C` is a char.
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct Sep<T, const C: char>(Vec<T>);

impl<T: FromStr, const C: char> FromStr for Sep<T, C> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(C)
            .map(T::from_str)
            .collect::<Result<Vec<_>, Self::Err>>()?
            .pipe(Sep::from)
            .pipe(Ok)
    }
}

impl<T: ToString, const C: char> ToString for Sep<T, C> {
    fn to_string(&self) -> String {
        self.0.iter().map(ToString::to_string).collect()
    }
}

impl<V: Into<Vec<T>>, T, const C: char> From<V> for Sep<T, C> {
    fn from(inner: V) -> Self {
        Sep(inner.into())
    }
}

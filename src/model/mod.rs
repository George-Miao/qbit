//! Model types used in the API.

use std::{
    fmt::{Display, Write},
    path::PathBuf,
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use tap::Pipe;

mod_use::mod_use![app, log, sync, torrent, transfer, search];

/// Username and password used to authenticate with qBittorrent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    username: String,
    password: String,
}

impl Credential {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }

    /// Return a dummy credential when you passed in the cookie instead of
    /// actual credential.
    pub fn dummy() -> Self {
        Self {
            username: "".to_owned(),
            password: "".to_owned(),
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.username.is_empty() && self.password.is_empty()
    }
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
    /// Number of seeds for current torrent, as reported by the tracker
    pub num_seeds: i64,
    /// Number of leeches for current torrent, as reported by the tracker
    pub num_leeches: i64,
    /// Number of completed downloads for current torrent, as reported by the
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

/// A wrapper around `str` that ensures the string is non-empty.
pub struct NonEmptyStr<T>(T);

impl<T: AsRef<str>> NonEmptyStr<T> {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn new(s: T) -> Option<Self> {
        if s.as_ref().is_empty() {
            None
        } else {
            Some(NonEmptyStr(s))
        }
    }
}

impl<T: Display, const C: char> Display for Sep<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_slice() {
            [] => Ok(()),
            [x] => x.fmt(f),
            [x, xs @ ..] => {
                x.fmt(f)?;
                for x in xs {
                    f.write_char(C)?;
                    x.fmt(f)?;
                }
                Ok(())
            }
        }
    }
}

impl<V: Into<Vec<T>>, T, const C: char> From<V> for Sep<T, C> {
    fn from(inner: V) -> Self {
        Sep(inner.into())
    }
}

#[test]
fn test_sep() {
    let sep = Sep::<u8, '|'>::from(vec![1, 2, 3]);
    assert_eq!(sep.to_string(), "1|2|3");

    let sep = Sep::<u8, '\n'>::from(vec![1, 2, 3]);
    assert_eq!(sep.to_string(), "1\n2\n3");

    let sep = Sep::<u8, '|'>::from(vec![1]);
    assert_eq!(sep.to_string(), "1");

    let sep = Sep::<u8, '|'>::from(vec![]);
    assert_eq!(sep.to_string(), "");
}

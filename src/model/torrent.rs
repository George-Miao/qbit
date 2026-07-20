use std::fmt::{Debug, Display};

use serde::Serialize;
use serde_with::{SerializeDisplay, skip_serializing_none};

use crate::{client::Url, model::Sep};

/// State filter used when retrieving the torrent list.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TorrentFilter {
    /// Return all torrents.
    All,
    /// Return torrents that are downloading.
    Downloading,
    /// Return torrents that have completed downloading.
    Completed,
    /// Return paused torrents.
    Paused,
    /// Return torrents that are transferring data.
    Active,
    /// Return torrents that are not transferring data.
    Inactive,
    /// Return resumed torrents.
    Resumed,
    /// Return stalled torrents.
    Stalled,
    /// Return stalled uploading torrents.
    StalledUploading,
    /// Return stalled downloading torrents.
    StalledDownloading,
    /// Return torrents in an error state.
    Errored,
}

/// Summary information for a torrent in the torrent list.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Torrent {
    /// Time (Unix Epoch) when the torrent was added to the client
    pub added_on: Option<i64>,
    /// Amount of data left to download (bytes)
    pub amount_left: Option<i64>,
    /// Whether this torrent is managed by Automatic Torrent Management
    pub auto_tmm: Option<bool>,
    /// Percentage of file pieces currently available
    pub availability: Option<f64>,
    /// Category of the torrent
    pub category: Option<String>,
    /// Amount of transfer data completed (bytes)
    pub comment: Option<String>,
    /// Comment for torrent
    pub completed: Option<i64>,
    /// Time (Unix Epoch) when the torrent completed
    pub completion_on: Option<i64>,
    /// Absolute path of torrent content (root path for multifile torrents,
    /// absolute file path for singlefile torrents)
    pub content_path: Option<String>,
    /// Torrent download speed limit (bytes/s). `-1` if unlimited.
    pub dl_limit: Option<i64>,
    /// Torrent download speed (bytes/s)
    pub dlspeed: Option<i64>,
    /// Torrent download path
    pub download_path: Option<String>,
    /// Amount of data downloaded
    pub downloaded: Option<i64>,
    /// Amount of data downloaded this session
    pub downloaded_session: Option<i64>,
    /// Torrent ETA (seconds)
    pub eta: Option<i64>,
    /// True if first last piece are prioritized
    pub f_l_piece_prio: Option<bool>,
    /// True if force start is enabled for this torrent
    pub force_start: Option<bool>,
    /// Whether the torrent has metadata
    pub has_metadata: Option<bool>,
    /// Torrent hash
    pub hash: Option<String>,
    /// Inactive seeding time limit
    pub inactive_seeding_time_limit: Option<i64>,
    /// Torrent infohash v1
    pub infohash_v1: Option<String>,
    /// Torrent infohash v2
    pub infohash_v2: Option<String>,
    /// Last time (Unix Epoch) when a chunk was downloaded/uploaded
    pub last_activity: Option<i64>,
    /// Magnet URI corresponding to this torrent
    pub magnet_uri: Option<String>,
    /// Maximum inactive seeding time
    pub max_inactive_seeding_time: Option<i64>,
    /// Maximum share ratio until torrent is stopped from seeding/uploading
    pub max_ratio: Option<f64>,
    /// Maximum seeding time (seconds) until torrent is stopped from seeding
    pub max_seeding_time: Option<i64>,
    /// Torrent name
    pub name: Option<String>,
    /// Number of seeds in the swarm
    pub num_complete: Option<i64>,
    /// Number of leechers in the swarm
    pub num_incomplete: Option<i64>,
    /// Number of leechers connected to
    pub num_leechs: Option<i64>,
    /// Number of seeds connected to
    pub num_seeds: Option<i64>,
    /// Torrent popularity
    pub popularity: Option<f64>,
    /// Torrent priority. Returns -1 if queuing is disabled or torrent is in
    /// seed mode
    pub priority: Option<i64>,
    /// Whether this torrent is private
    pub private: Option<bool>,
    /// Torrent progress (percentage/100)
    pub progress: Option<f64>,
    /// Torrent share ratio. Max ratio value: 9999.
    pub ratio: Option<f64>,
    /// Maximum share ratio before the torrent stops seeding.
    pub ratio_limit: Option<f64>,
    /// Torrent reannounce interval
    pub reannounce: Option<i64>,
    /// Root folder of the torrent
    pub root_path: Option<String>,
    /// Path where this torrent's data is stored
    pub save_path: Option<String>,
    /// Torrent elapsed time while complete (seconds)
    pub seeding_time: Option<i64>,
    /// seeding_time_limit is a per torrent setting, when Automatic Torrent
    /// Management is disabled, furthermore then max_seeding_time is set to
    /// seeding_time_limit for this torrent. If Automatic Torrent Management
    /// is enabled, the value is -2. And if max_seeding_time is unset it
    /// have a default value -1.
    pub seeding_time_limit: Option<i64>,
    /// Time (Unix Epoch) when this torrent was last seen complete
    pub seen_complete: Option<i64>,
    /// True if sequential download is enabled
    pub seq_dl: Option<bool>,
    /// Total size (bytes) of files selected for download
    pub size: Option<i64>,
    /// Torrent state. See table here below for the possible values
    pub state: Option<State>,
    /// True if super seeding is enabled
    pub super_seeding: Option<bool>,
    /// Comma-concatenated tag list of the torrent
    pub tags: Option<String>,
    /// Total active time (seconds)
    pub time_active: Option<i64>,
    /// Total size (bytes) of all file in this torrent (including unselected
    /// ones)
    pub total_size: Option<i64>,
    /// The first tracker with working status. Returns empty String if no
    /// tracker is working.
    pub tracker: Option<String>,
    /// Torrent upload speed limit (bytes/s). `-1` if unlimited.
    pub up_limit: Option<i64>,
    /// Amount of data uploaded
    pub uploaded: Option<i64>,
    /// Amount of data uploaded this session
    pub uploaded_session: Option<i64>,
    /// Torrent upload speed (bytes/:,)
    pub upspeed: Option<i64>,
}

/// Current transfer state of a torrent.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum State {
    /// Some error occurred, applies to paused torrents
    #[serde(rename = "error")]
    Error,
    /// Torrent data files is missing
    #[serde(rename = "missingFiles")]
    MissingFiles,
    /// Torrent is being seeded and data is being transferred
    #[serde(rename = "uploading")]
    Uploading,
    /// Torrent is paused and has finished downloading,
    /// stoppedUP is new name in qBit5+
    #[serde(rename = "pausedUP", alias = "stoppedUP")]
    PausedUP,
    /// Queuing is enabled and torrent is queued for upload
    #[serde(rename = "queuedUP")]
    QueuedUP,
    /// Torrent is being seeded, but no connection were made
    #[serde(rename = "stalledUP")]
    StalledUP,
    /// Torrent has finished downloading and is being checked
    #[serde(rename = "checkingUP")]
    CheckingUP,
    /// Torrent is forced to uploading and ignore queue limit
    #[serde(rename = "forcedUP")]
    ForcedUP,
    /// Torrent is allocating disk space for download
    #[serde(rename = "allocating")]
    Allocating,
    /// Torrent is being downloaded and data is being transferred
    #[serde(rename = "downloading")]
    Downloading,
    /// Torrent has just started downloading and is fetching metadata
    #[serde(rename = "metaDL")]
    MetaDL,
    /// Torrent is paused and has NOT finished downloading,
    /// stoppedDL is new name in qBit5+
    #[serde(rename = "pausedDL", alias = "stoppedDL")]
    PausedDL,
    /// Queuing is enabled and torrent is queued for download
    #[serde(rename = "queuedDL")]
    QueuedDL,
    /// Torrent is being downloaded, but no connection were made
    #[serde(rename = "stalledDL")]
    StalledDL,
    /// Same as checkingUP, but torrent has NOT finished downloading
    #[serde(rename = "checkingDL")]
    CheckingDL,
    /// Torrent is forced to downloading to ignore queue limit
    #[serde(rename = "forcedDL")]
    ForcedDL,
    /// Checking resume data on qBt startup
    #[serde(rename = "checkingResumeData")]
    CheckingResumeData,
    /// Torrent is moving to another location
    #[serde(rename = "moving")]
    Moving,
    /// Unknown status
    #[serde(rename = "unknown")]
    Unknown,
}

/// General properties of a torrent.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct TorrentProperty {
    /// Torrent save path
    pub save_path: Option<String>,
    /// Torrent creation date (Unix timestamp)
    pub creation_date: Option<i64>,
    /// Torrent piece size (bytes)
    pub piece_size: Option<i64>,
    /// Torrent comment
    pub comment: Option<String>,
    /// Total data wasted for torrent (bytes)
    pub total_wasted: Option<i64>,
    /// Total data uploaded for torrent (bytes)
    pub total_uploaded: Option<i64>,
    /// Total data uploaded this session (bytes)
    pub total_uploaded_session: Option<i64>,
    /// Total data downloaded for torrent (bytes)
    pub total_downloaded: Option<i64>,
    /// Total data downloaded this session (bytes)
    pub total_downloaded_session: Option<i64>,
    /// Torrent upload limit (bytes/s)
    pub up_limit: Option<i64>,
    /// Torrent download limit (bytes/s)
    pub dl_limit: Option<i64>,
    /// Torrent elapsed time (seconds)
    pub time_elapsed: Option<i64>,
    /// Torrent elapsed time while complete (seconds)
    pub seeding_time: Option<i64>,
    /// Torrent connection count
    pub nb_connections: Option<i64>,
    /// Torrent connection count limit
    pub nb_connections_limit: Option<i64>,
    /// Torrent share ratio
    pub share_ratio: Option<f64>,
    /// When this torrent was added (unix timestamp)
    pub addition_date: Option<i64>,
    /// Torrent completion date (unix timestamp)
    pub completion_date: Option<i64>,
    /// Number of distributed copies of the torrent's selected files.
    /// Added in qBittorrent 5.2.0 (Web API v2.15.1).
    pub availability: Option<f64>,
    /// Torrent creator
    pub created_by: Option<String>,
    /// Torrent average download speed (bytes/second)
    pub dl_speed_avg: Option<i64>,
    /// Torrent download speed (bytes/second)
    pub dl_speed: Option<i64>,
    /// Torrent ETA (seconds)
    pub eta: Option<i64>,
    /// Last seen complete date (unix timestamp)
    pub last_seen: Option<i64>,
    /// Number of peers connected to
    pub peers: Option<i64>,
    /// Number of peers in the swarm
    pub peers_total: Option<i64>,
    /// Number of pieces owned
    pub pieces_have: Option<i64>,
    /// Number of pieces of the torrent
    pub pieces_num: Option<i64>,
    /// Number of seconds until the next announce
    pub reannounce: Option<i64>,
    /// Number of seeds connected to
    pub seeds: Option<i64>,
    /// Number of seeds in the swarm
    pub seeds_total: Option<i64>,
    /// Torrent total size (bytes)
    pub total_size: Option<i64>,
    /// Torrent average upload speed (bytes/second)
    pub up_speed_avg: Option<i64>,
    /// Torrent upload speed (bytes/second)
    pub up_speed: Option<i64>,
}

/// A web seed configured for a torrent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WebSeed {
    /// Web seed URL
    pub url: Url,
}

/// Information about a file contained in a torrent.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TorrentContent {
    /// File index
    pub index: u64,
    /// File name (including relative path),
    pub name: String,
    /// File size (bytes),
    pub size: u64,
    /// File progress (percentage/100),
    pub progress: f64,
    /// File priority. See possible values here below,
    pub priority: Priority,
    /// True if file is seeding/complete,
    pub is_seed: Option<bool>,
    /// The first number is the starting piece index and the second number is
    /// the ending piece index (inclusive),
    #[serde(default)]
    pub piece_range: Vec<i64>,
    /// Percentage of file pieces currently available (percentage/100),
    #[serde(default)]
    pub availability: f64,
}

/// Download priority assigned to a torrent file.
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
#[repr(u8)]
pub enum Priority {
    /// Do not download
    DoNotDownload = 0,
    /// Normal priority
    Normal        = 1,
    /// Mixed
    Mixed         = 4,
    /// High priority
    High          = 6,
    /// Maximal priority
    Maximal       = 7,
}

/// Download state of a torrent piece.
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
#[repr(u8)]
pub enum PieceState {
    /// Not downloaded yet
    NotDownloaded = 0,
    /// Now downloading
    Downloading   = 1,
    /// Already downloaded
    Downloaded    = 2,
}

/// `|` separeated list of hash values or `all`
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay)]
pub enum Hashes {
    /// A list of torrent hashes separated by `|`
    Hashes(Sep<String, '|'>),
    /// All torrents
    All,
}

impl<V: Into<Vec<String>>> From<V> for Hashes {
    fn from(hashes: V) -> Self {
        Hashes::Hashes(Sep::from(hashes))
    }
}

impl Display for Hashes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hashes::Hashes(hashes) => write!(f, "{}", hashes),
            Hashes::All => write!(f, "all"),
        }
    }
}

/// Parameters for filtering, sorting, and paginating the torrent list.
#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
#[cfg_attr(
    feature = "builder",
    builder(field_defaults(default, setter(strip_option)))
)]
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
#[skip_serializing_none]
pub struct GetTorrentListArg {
    /// Filter torrent list by state. Allowed state filters: `all`,
    /// `downloading`, `seeding`, `completed`, `paused`, `active`, `inactive`,
    /// `resumed`, `stalled`, `stalled_uploading`, `stalled_downloading`,
    /// `errored`
    pub filter: Option<TorrentFilter>,
    /// Get torrents with the given category (empty string means "without category"; no "category" parameter means "any category" <- broken until [#11748](https://github.com/qbittorrent/qBittorrent/issues/11748) is resolved). Remember to URL-encode the category name. For example, `My category` becomes `My%20category`
    pub category: Option<String>,
    /// Get torrents with the given tag (empty string means "without tag"; no
    /// "tag" parameter means "any tag". Remember to URL-encode the category
    /// name. For example, `My tag` becomes `My%20tag`
    pub tag: Option<String>,
    /// Sort torrents by given key. They can be sorted using any field of the
    /// response's JSON array (which are documented below) as the sort key.
    pub sort: Option<String>,
    /// Enable reverse sorting. Defaults to `false`
    pub reverse: Option<bool>,
    /// Limit the number of torrents returned
    pub limit: Option<u64>,
    /// Set offset (if less than 0, offset from end)
    pub offset: Option<i64>,
    /// Filter by hashes. Can contain multiple hashes separated by `\|`
    pub hashes: Option<String>,
}

/// Source used to add one or more torrents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum TorrentSource {
    /// URLs
    Urls {
        /// URLs separated by newlines in the request.
        urls: Sep<Url, '\n'>,
    },
    /// Torrent files
    TorrentFiles {
        /// Torrent files included as multipart form fields.
        torrents: Vec<TorrentFile>,
    },
}
/// Torrent file
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TorrentFile {
    /// Name to use for the multipart file field.
    pub filename: String,
    /// Raw torrent file contents.
    pub data: Vec<u8>,
}
impl Default for TorrentSource {
    fn default() -> Self {
        TorrentSource::Urls {
            urls: Sep::from(vec![]),
        }
    }
}
fn is_torrent_files(source: &TorrentSource) -> bool {
    matches!(source, TorrentSource::TorrentFiles { .. })
}
/// Parameters used when adding one or more torrents.
#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
#[cfg_attr(
    feature = "builder",
    builder(field_defaults(default, setter(strip_option)))
)]
#[derive(Debug, Clone, PartialEq, serde::Serialize, Default)]
#[skip_serializing_none]
pub struct AddTorrentArg {
    /// URLs or torrent files to add.
    #[serde(flatten)]
    #[cfg_attr(feature = "builder", builder(!default, setter(!strip_option)))]
    #[serde(skip_serializing_if = "is_torrent_files")]
    pub source: TorrentSource,
    /// Download folder
    #[serde(skip_serializing_if = "Option::is_none")]
    pub savepath: Option<String>,
    /// Cookie sent to download the .torrent file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookie: Option<String>,
    /// Download torrent using a search plugin (added in qBittorrent 5.2.0, Web
    /// API v2.13.1). Specify the search plugin name to use for downloading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloader: Option<String>,
    /// Category for the torrent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Tags for the torrent, split by ','
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// Skip hash checking. Possible values are `true`, `false` (default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_checking: Option<String>,

    /// Add torrents in the paused state. Possible values are `true`, `false`
    /// (default).
    ///
    /// This is the field name used by qBittorrent before 5.0; version 5.0+
    /// renamed it to [`stopped`](Self::stopped). You only need to set one of
    /// the two — [`add_torrent`](crate::Qbit::add_torrent) copies whichever is
    /// set into the other so the request is honored regardless of the server
    /// version. If both are set, they are sent as-is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<String>,

    /// Add torrents in the stopped state. Possible values are `true`, `false`
    /// (default).
    ///
    /// This is the field name used by qBittorrent 5.0+; it is the successor of
    /// [`paused`](Self::paused). You only need to set one of the two —
    /// [`add_torrent`](crate::Qbit::add_torrent) copies whichever is set into
    /// the other so the request is honored regardless of the server version.
    /// If both are set, they are sent as-is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped: Option<String>,

    /// Create the root folder. Possible values are `true`, `false`, unset
    /// (default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_folder: Option<String>,

    /// Content layout of the torrent, controlling how the downloaded files are
    /// placed on disk. Unset uses the server's default. Supersedes
    /// [`root_folder`](Self::root_folder) on qBittorrent 4.3.2+.
    #[serde(rename = "contentLayout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_layout: Option<ContentLayout>,

    /// Rename torrent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,

    /// Set torrent upload speed limit. Unit in bytes/second
    #[serde(rename = "upLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_limit: Option<i64>,

    /// Set torrent download speed limit. Unit in bytes/second
    #[serde(rename = "dlLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_limit: Option<i64>,

    /// Set torrent share ratio limit
    #[serde(rename = "ratioLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratio_limit: Option<f64>,

    /// Set torrent seeding time limit. Unit in minutes
    #[serde(rename = "seedingTimeLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeding_time_limit: Option<i64>,

    /// Whether Automatic Torrent Management should be used
    #[serde(rename = "autoTMM")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_torrent_management: Option<bool>,

    /// Enable sequential download. Possible values are `true`, `false`
    /// (default)
    #[serde(rename = "sequentialDownload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequential_download: Option<String>,

    /// Prioritize download first last piece. Possible values are `true`,
    /// `false` (default)
    #[serde(rename = "firstLastPiecePrio")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_last_piece_priority: Option<String>,
}

/// How the files of a torrent are laid out on disk when added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContentLayout {
    /// Use the layout as defined by the torrent itself.
    Original,
    /// Always create a subfolder containing the torrent's files.
    Subfolder,
    /// Never create a subfolder, placing files directly in the save path.
    NoSubfolder,
}

#[cfg(test)]
mod content_layout_tests {
    use super::ContentLayout;

    #[test]
    fn serializes_to_api_names() {
        for (variant, expected) in [
            (ContentLayout::Original, "Original"),
            (ContentLayout::Subfolder, "Subfolder"),
            (ContentLayout::NoSubfolder, "NoSubfolder"),
        ] {
            assert_eq!(serde_json::to_value(variant).unwrap(), expected);
        }
    }
}

#[cfg(test)]
mod torrent_source_tests {
    use super::{AddTorrentArg, TorrentFile, TorrentSource};

    /// Regression test for <https://github.com/George-Miao/qbit/issues/11>:
    /// adding a torrent from disk used to fail with `unsupported value`
    /// because the binary payload was fed to `serde_urlencoded`. The raw file
    /// bytes are now sent as a separate multipart part, so the arg itself must
    /// serialize without the `source`/`torrents` field.
    #[test]
    fn torrent_files_source_is_skipped_on_serialization() {
        let arg = AddTorrentArg {
            source: TorrentSource::TorrentFiles {
                torrents: vec![TorrentFile {
                    filename: "test.torrent".to_owned(),
                    // Non-UTF-8 bytes that `serde_urlencoded` cannot encode.
                    data: vec![0x64, 0x38, 0x00, 0xff, 0xfe],
                }],
            },
            ..Default::default()
        };

        let value = serde_json::to_value(&arg).unwrap();
        let obj = value.as_object().expect("should serialize to an object");
        assert!(
            !obj.contains_key("torrents") && !obj.contains_key("source"),
            "binary torrent payload must not be serialized into the form: {obj:?}"
        );

        // The trimmed-down arg is now safe to urlencode.
        serde_urlencoded::to_string(&arg).expect("arg without binary source must urlencode");
    }
}

/// Per-torrent share limits.
#[cfg_attr(feature = "builder", derive(typed_builder::TypedBuilder))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTorrentSharedLimitArg {
    /// Torrents whose limits are changed.
    #[cfg_attr(feature = "builder", builder(setter(into)))]
    pub hashes: Hashes,
    /// Share ratio limit.
    #[cfg_attr(feature = "builder", builder(default, setter(strip_option)))]
    pub ratio_limit: Option<RatioLimit>,
    /// Total seeding time limit.
    #[cfg_attr(feature = "builder", builder(default, setter(strip_option)))]
    pub seeding_time_limit: Option<SeedingTimeLimit>,
    /// Inactive seeding time limit.
    #[cfg_attr(feature = "builder", builder(default, setter(strip_option)))]
    pub inactive_seeding_time_limit: Option<SeedingTimeLimit>,
}

/// Share ratio limit for a torrent.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RatioLimit {
    /// Use the global share ratio limit.
    Global,
    /// Disable the share ratio limit.
    NoLimit,
    /// Use the specified share ratio limit.
    Limited(f64),
}

impl Serialize for RatioLimit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Global => serializer.serialize_i64(-2),
            Self::NoLimit => serializer.serialize_i64(-1),
            Self::Limited(limit) => serializer.serialize_f64(*limit),
        }
    }
}

/// Seeding time limit for a torrent.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SeedingTimeLimit {
    /// Use the global seeding time limit.
    Global,
    /// Disable the seeding time limit.
    NoLimit,
    /// Number of minutes
    Limited(u64),
}

impl Serialize for SeedingTimeLimit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Global => serializer.serialize_i64(-2),
            Self::NoLimit => serializer.serialize_i64(-1),
            Self::Limited(limit) => serializer.serialize_u64(*limit),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct HashArg<'a> {
    hash: &'a str,
}

impl<'a> HashArg<'a> {
    pub(crate) fn new(hash: &'a str) -> Self {
        Self { hash }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct HashesArg {
    hashes: Hashes,
}

impl HashesArg {
    pub(crate) fn new(hashes: impl Into<Hashes> + Send + Sync) -> Self {
        Self {
            hashes: hashes.into(),
        }
    }
}

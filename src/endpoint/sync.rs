use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{Qbit, Result, ext::*, model::*};

impl Qbit {
    /// Return main-data changes since the supplied response ID.
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

    /// Return peer-data changes for a torrent since the supplied response ID.
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
}

use std::borrow::Borrow;

use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{Qbit, Result, model::*};

impl Qbit {
    /// Return application log entries matching the requested filters.
    pub async fn get_logs(&self, arg: impl Borrow<GetLogsArg> + Send + Sync) -> Result<Vec<Log>> {
        self.get_with("log/main", arg.borrow())
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Return peer log entries newer than the requested message ID.
    pub async fn get_peer_logs(
        &self,
        last_known_id: impl Into<Option<i64>> + Send + Sync,
    ) -> Result<Vec<PeerLog>> {
        #[derive(Serialize)]
        #[skip_serializing_none]
        struct Arg {
            last_known_id: Option<i64>,
        }

        self.get_with(
            "log/peers",
            &Arg {
                last_known_id: last_known_id.into(),
            },
        )
        .await?
        .json()
        .await
        .map_err(Into::into)
    }
}

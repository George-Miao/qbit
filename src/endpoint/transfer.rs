use serde::Serialize;

use crate::{Error, Qbit, Result, ext::*, model::*};

impl Qbit {
    /// Return the global transfer information shown in the qBittorrent status
    /// bar.
    pub async fn get_transfer_info(&self) -> Result<TransferInfo> {
        self.get("transfer/info")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Return whether alternative speed limits are enabled.
    pub async fn get_speed_limits_mode(&self) -> Result<bool> {
        self.get("transfer/speedLimitsMode")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| match s.as_str() {
                "0" => Ok(false),
                "1" => Ok(true),
                _ => Err(Error::BadResponse {
                    explain: "Received non-number response body on `transfer/speedLimitsMode`",
                }),
            })
    }

    /// Toggle alternative speed limits.
    pub async fn toggle_speed_limits_mode(&self) -> Result<()> {
        self.post("transfer/toggleSpeedLimitsMode").await?.end()
    }

    /// Get global and alternative speed limits (KiB/s, -1 = unlimited).
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.16.0).
    pub async fn get_speed_limits(&self) -> Result<SpeedLimits> {
        self.get("transfer/getSpeedLimits")
            .await?
            .json()
            .await
            .map_err(Into::into)
    }

    /// Set global and alternative speed limits (KiB/s, -1 = unlimited).
    ///
    /// Added in qBittorrent 5.2.0 (Web API v2.16.0).
    pub async fn set_speed_limits(&self, limits: &SpeedLimits) -> Result<()> {
        self.post_with("transfer/setSpeedLimits", limits)
            .await?
            .end()
    }

    /// Return the global download limit in bytes per second.
    pub async fn get_download_limit(&self) -> Result<u64> {
        self.get("transfer/downloadLimit")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| {
                s.parse().map_err(|_| Error::BadResponse {
                    explain: "Received non-number response body on `transfer/downloadLimit`",
                })
            })
    }

    /// Set the global download limit in bytes per second.
    pub async fn set_download_limit(&self, limit: u64) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            limit: u64,
        }

        self.post_with("transfer/setDownloadLimit", &Arg { limit })
            .await?
            .end()
    }

    /// Return the global upload limit in bytes per second.
    pub async fn get_upload_limit(&self) -> Result<u64> {
        self.get("transfer/uploadLimit")
            .await?
            .text()
            .await
            .map_err(Into::into)
            .and_then(|s| {
                s.parse().map_err(|_| Error::BadResponse {
                    explain: "Received non-number response body on `transfer/uploadLimit`",
                })
            })
    }

    /// Set the global upload limit in bytes per second.
    pub async fn set_upload_limit(&self, limit: u64) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            limit: u64,
        }

        self.post_with("transfer/setUploadLimit", &Arg { limit })
            .await?
            .end()
    }

    /// Ban the supplied peers, identified by IP address and optional port.
    pub async fn ban_peers(&self, peers: impl Into<Sep<String, '|'>> + Send + Sync) -> Result<()> {
        #[derive(Serialize)]
        struct Arg {
            peers: String,
        }

        self.post_with(
            "transfer/banPeers",
            &Arg {
                peers: peers.into().to_string(),
            },
        )
        .await?
        .end()
    }
}

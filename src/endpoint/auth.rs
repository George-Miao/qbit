use tap::Pipe;
use tracing::{debug, trace};

use crate::{
    ApiError, Error, Qbit, Result,
    client::{CheckError, Method, StatusCode},
    ext::*,
};

impl Qbit {
    /// Return the authentication cookie currently used by this client.
    pub async fn get_cookie(&self) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .as_header()
            .1
            .map(ToOwned::to_owned)
    }

    /// Log out of the qBittorrent WebUI session.
    pub async fn logout(&self) -> Result<()> {
        self.get("auth/logout").await?.end()
    }

    /// Log in to qBittorrent.
    ///
    /// Set force to `true` to force re-login regardless if cookie is already
    /// set.
    pub async fn login(&self, force: bool) -> Result<()> {
        let re_login = force || { self.state().as_header().1.is_none() };
        if re_login {
            let credential = match self.state().as_credential() {
                Some(credential) => credential.clone(),
                None => {
                    trace!("API key or cookie auth in use, skipping login");
                    return Ok(());
                }
            };
            debug!("Cookie not found, logging in");
            self.client
                .request(Method::POST, self.url("auth/login"))
                .check()?
                .pipe(|req| req.form(&credential))
                .check()?
                .send()
                .await?
                .map_status(|code| match code as _ {
                    StatusCode::FORBIDDEN => Some(Error::ApiError(ApiError::IpBanned)),
                    StatusCode::UNAUTHORIZED => Some(Error::ApiError(ApiError::BadCredentials)),
                    _ => None,
                })?
                .extract::<Cookie>()?
                .pipe(|Cookie(cookie)| self.state.lock().unwrap().add_cookie(cookie));

            debug!("Log in success");
        } else {
            trace!("Already logged in, skipping");
        }

        Ok(())
    }
}

use http_client::{
    http_types::{headers::SET_COOKIE, StatusCode},
    Response,
};

use crate::{ApiError, Error};

pub trait FromResponse {
    fn from_response(response: &Response) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ResponseExt: Sized {
    fn extract<T: FromResponse>(&self) -> Result<T, Error>;

    fn map_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self, Error>;
}

impl ResponseExt for Response {
    fn extract<T: FromResponse>(&self) -> Result<T, Error> {
        T::from_response(self)
    }

    fn map_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self, Error> {
        let status = self.status();

        if status.is_success() {
            Ok(self)
        } else {
            match f(status) {
                Some(err) => Err(err),
                None => match status {
                    StatusCode::Forbidden => Err(Error::ApiError(ApiError::NotLoggedIn)),
                    _ => Ok(self),
                },
            }
        }
    }
}

/// Handle 404 returned by APIs with torrent hash as a parameter
pub const TORRENT_NOT_FOUND: fn(StatusCode) -> Option<Error> = |s| {
    if s == StatusCode::NotFound {
        Some(Error::ApiError(ApiError::TorrentNotFound))
    } else {
        None
    }
};

pub struct Cookie(pub String);

impl FromResponse for Cookie {
    fn from_response(response: &Response) -> Result<Self, Error> {
        let cookie = response
            .header(SET_COOKIE)
            .ok_or(Error::BadResponse {
                explain: "Failed to extract cookie from response",
            })?
            .as_str()
            .to_owned();
        Ok(Self(cookie))
    }
}

use http_client::{
    http_types::{headers::SET_COOKIE, StatusCode},
    Response,
};
use tap::Pipe;

use crate::{ApiError, Error, Result};

pub trait FromResponse {
    fn from_response(response: &Response) -> Result<Self>
    where
        Self: Sized;
}

pub struct Cookie(pub String);

impl FromResponse for Cookie {
    fn from_response(response: &Response) -> Result<Self> {
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

impl FromResponse for () {
    fn from_response(_: &Response) -> Result<Self> {
        Ok(())
    }
}

pub trait ResponseExt: Sized {
    fn extract<T: FromResponse>(&self) -> Result<T>;

    fn map_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self>;

    fn end<T: FromResponse>(self) -> Result<T>;
}

impl ResponseExt for Response {
    fn extract<T: FromResponse>(&self) -> Result<T> {
        T::from_response(self)
    }

    fn map_status<F: FnOnce(StatusCode) -> Option<Error>>(self, f: F) -> Result<Self> {
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

    fn end<T: FromResponse>(self) -> Result<T> {
        self.map_status(|c| Error::UnknownHttpCode(c).pipe(Some))
            .and_then(|b| T::from_response(&b).map_err(Into::into))
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

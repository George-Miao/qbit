use reqwest::{header::SET_COOKIE, Response, StatusCode};
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
            .headers()
            .get(SET_COOKIE)
            .ok_or(Error::BadResponse {
                explain: "Failed to extract cookie from response",
            })?
            .to_str()
            .map_err(|_| Error::NonAsciiHeader)?
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
                    StatusCode::FORBIDDEN => Err(Error::ApiError(ApiError::NotLoggedIn)),
                    _ => Ok(self),
                },
            }
        }
    }

    fn end<T: FromResponse>(self) -> Result<T> {
        self.map_status(|c| Error::UnknownHttpCode(c).pipe(Some))
            .and_then(|b| T::from_response(&b))
    }
}

/// Handle 404 returned by APIs with torrent hash as a parameter
pub const TORRENT_NOT_FOUND: fn(StatusCode) -> Option<Error> = |s| {
    if s == StatusCode::NOT_FOUND {
        Some(Error::ApiError(ApiError::TorrentNotFound))
    } else {
        None
    }
};

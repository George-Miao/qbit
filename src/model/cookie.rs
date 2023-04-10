use http_client::{
    http_types::{cookies::Cookie as HttpCookie, headers::SET_COOKIE},
    Response,
};
use tap::Pipe;

use crate::{model::FromResponse, Error};

pub struct Cookie {
    pub cookie: HttpCookie<'static>,
}

impl FromResponse for Cookie {
    fn from_response(response: &Response) -> Result<Self, Error> {
        let cookie = response
            .header(SET_COOKIE)
            .ok_or(Error::BadResponse {
                explain: "Failed to extract cookie from response",
            })?
            .as_str()
            .to_owned()
            .pipe(HttpCookie::parse)
            .map_err(|_| Error::BadResponse {
                explain: "API returned invalid cookie",
            })?;
        Ok(Self { cookie })
    }
}

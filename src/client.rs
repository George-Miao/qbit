#![allow(unused_imports)]

use crate::Result;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "reqwest", feature = "cyper"))] {
        compile_error!("The 'reqwest' and 'cyper' features cannot be enabled at the same time. To use `cyper`, disable default feature first.");
    } else if #[cfg(feature = "reqwest")] {
        pub(crate) use reqwest::{Client, Error, Method, Response, StatusCode, Url, RequestBuilder, get, header, multipart};
    } else if #[cfg(feature = "cyper")] {
        pub(crate) use cyper::{Client, Response, Error, RequestBuilder, multipart};
        pub(crate) use url::Url;
        pub(crate) use http::{Method, StatusCode, header};
        pub(crate) use cyper_ext::*;
    } else {
        compile_error!("Either the 'reqwest' or 'compio' feature must be enabled");
    }
}

pub(crate) trait CheckError: Sized {
    type Ok;

    fn check(self) -> Result<Self::Ok>;
}

#[cfg(feature = "reqwest")]
impl CheckError for reqwest::RequestBuilder {
    type Ok = reqwest::RequestBuilder;

    #[inline(always)]
    fn check(self) -> Result<Self> {
        Ok(self)
    }
}

#[cfg(feature = "cyper")]
mod cyper_ext {
    use cyper::multipart::Part;
    use mime::FromStrError;

    use super::*;

    pub(crate) trait PartExt: Sized {
        fn mime_str(self, mime: &str) -> Result<Part, FromStrError>;
    }

    impl PartExt for multipart::Part {
        fn mime_str(self, mime: &str) -> Result<Part, FromStrError> {
            let mime = mime.parse()?;
            Ok(self.mime(mime))
        }
    }

    #[cfg(test)]
    pub(crate) async fn get<T: cyper::IntoUrl>(url: T) -> Result<Response, cyper::Error> {
        Client::new().get(url)?.send().await
    }

    impl<T> CheckError for cyper::Result<T> {
        type Ok = T;

        #[inline(always)]
        fn check(self) -> Result<T> {
            Ok(self?)
        }
    }
}

#![allow(private_interfaces, private_bounds)]

use std::{fmt::Debug, sync::Mutex};

use tap::Pipe;
use url::Url;

use crate::{Client, LoginState, Qbit, ext::Cookie, model::Credential};

pub struct QbitBuilder<C = (), R = (), E = ()> {
    credential: C,
    client: R,
    endpoint: E,
}

trait IntoLoginState {
    fn into_login_state(self) -> LoginState;
}

impl IntoLoginState for Cookie {
    fn into_login_state(self) -> LoginState {
        LoginState::CookieProvided { cookie: self.0 }
    }
}

impl IntoLoginState for Credential {
    fn into_login_state(self) -> LoginState {
        LoginState::NotLoggedIn { credential: self }
    }
}

impl QbitBuilder {
    /// Creates a new `QbitBuilder` with default values.
    pub fn new() -> Self {
        QbitBuilder {
            credential: (),
            client: (),
            endpoint: (),
        }
    }
}

impl Default for QbitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, R, E> QbitBuilder<C, R, E> {
    /// Sets the HTTP client for the `Qbit` instance.
    ///
    /// - When `reqwest` feature is enabled (by default), this method accepts a
    /// `reqwest::Client`.
    /// - When `cyper` feature is enabled, this method accepts a
    ///   `cyper::Client`.
    pub fn client(self, client: Client) -> QbitBuilder<C, Client, E> {
        QbitBuilder {
            credential: self.credential,
            client,
            endpoint: self.endpoint,
        }
    }

    /// Sets the cookie for authentication.
    ///
    /// Note that if you have already set the credential, this method will
    /// overwrite the credential and use the cookie instead. The builder
    /// will use the latest provided credential for authentication.
    #[allow(private_interfaces)]
    pub fn cookie(self, cookie: impl Into<String>) -> QbitBuilder<Cookie, R, E> {
        QbitBuilder {
            credential: Cookie(cookie.into()),
            client: self.client,
            endpoint: self.endpoint,
        }
    }

    /// Sets the username-password credentials for authentication.
    ///
    /// Note that if you have already set the cookie, this method will overwrite
    /// the cookie and use the credential instead. The builder will use the
    /// latest provided credential for authentication.
    pub fn credential(self, credential: Credential) -> QbitBuilder<Credential, R, E> {
        QbitBuilder {
            credential,
            client: self.client,
            endpoint: self.endpoint,
        }
    }

    /// Sets the endpoint URL for the qBittorrent Web API.
    pub fn endpoint<U>(self, endpoint: U) -> QbitBuilder<C, R, U>
    where
        U: TryInto<Url>,
    {
        QbitBuilder {
            credential: self.credential,
            client: self.client,
            endpoint,
        }
    }
}

impl<C, U> QbitBuilder<C, Client, U>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    /// Builds the `Qbit` instance with the provided configuration and HTTP
    /// Client.
    pub fn build(self) -> Qbit {
        let endpoint = self.endpoint.try_into().expect("Invalid endpoint");
        let state = self.credential.into_login_state().pipe(Mutex::new);

        Qbit {
            client: self.client,
            endpoint,
            state,
        }
    }
}

impl<C, U> QbitBuilder<C, (), U>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    /// Builds the `Qbit` instance with the provided configuration and a default
    /// HTTP Client.
    pub fn build(self) -> Qbit {
        self.client(Client::new()).build()
    }
}

#[test]
fn test_builder() {
    QbitBuilder::new()
        .client(Client::new())
        .endpoint("http://localhost:8080")
        .credential(Credential::new("admin", "adminadmin"))
        .build();

    QbitBuilder::new()
        .endpoint("http://localhost:8080")
        .credential(Credential::new("admin", "adminadmin"))
        .build();

    QbitBuilder::new()
        .client(Client::new())
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .build();

    QbitBuilder::new()
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .build();
}

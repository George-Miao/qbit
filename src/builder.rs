#![allow(private_interfaces, private_bounds)]

use std::{fmt::Debug, sync::Mutex};

use reqwest::Client;
use tap::Pipe;
use url::Url;

use crate::{LoginState, Qbit, ext::Cookie, model::Credential};

pub struct QbitBuilder<C = (), R = (), E = (), B = ()> {
    credential: C,
    client: R,
    endpoint: E,
    basic_auth_credentials: B,
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
    pub fn new() -> Self {
        QbitBuilder {
            credential: (),
            client: (),
            endpoint: (),
            basic_auth_credentials: (),
        }
    }
}

impl Default for QbitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, R, E, B> QbitBuilder<C, R, E, B> {
    pub fn client(self, client: Client) -> QbitBuilder<C, Client, E, B> {
        QbitBuilder {
            credential: self.credential,
            client,
            endpoint: self.endpoint,
            basic_auth_credentials: self.basic_auth_credentials,
        }
    }

    #[allow(private_interfaces)]
    pub fn cookie(self, cookie: impl Into<String>) -> QbitBuilder<Cookie, R, E, B> {
        QbitBuilder {
            credential: Cookie(cookie.into()),
            client: self.client,
            endpoint: self.endpoint,
            basic_auth_credentials: self.basic_auth_credentials,
        }
    }

    pub fn credential(self, credential: Credential) -> QbitBuilder<Credential, R, E, B> {
        QbitBuilder {
            credential,
            client: self.client,
            endpoint: self.endpoint,
            basic_auth_credentials: self.basic_auth_credentials,
        }
    }

    pub fn endpoint<U>(self, endpoint: U) -> QbitBuilder<C, R, U, B>
    where
        U: TryInto<Url>,
    {
        QbitBuilder {
            credential: self.credential,
            client: self.client,
            endpoint,
            basic_auth_credentials: self.basic_auth_credentials,
        }
    }

    pub fn basic_auth_credentials(
        self,
        basic_auth_credentials: Option<Credential>,
    ) -> QbitBuilder<C, R, E, Option<Credential>> {
        QbitBuilder {
            credential: self.credential,
            client: self.client,
            endpoint: self.endpoint,
            basic_auth_credentials,
        }
    }
}

impl<C, U> QbitBuilder<C, reqwest::Client, U, Option<Credential>>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    pub fn build(self) -> Qbit {
        let endpoint = self.endpoint.try_into().expect("Invalid endpoint");
        let state = self.credential.into_login_state().pipe(Mutex::new);

        Qbit {
            client: self.client,
            endpoint,
            state,
            basic_auth_credentials: self.basic_auth_credentials,
        }
    }
}

impl<C, U> QbitBuilder<C, (), U, Option<Credential>>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    pub fn build(self) -> Qbit {
        self.client(reqwest::Client::new()).build()
    }
}

// No basic auth credential provided
impl<C, U> QbitBuilder<C, (), U, ()>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    pub fn build(self) -> Qbit {
        self.basic_auth_credentials(None).build()
    }
}

// TODO: How to factorize with previous one?
impl<C, U> QbitBuilder<C, reqwest::Client, U, ()>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    pub fn build(self) -> Qbit {
        self.basic_auth_credentials(None).build()
    }
}

#[test]
fn test_builder() {
    QbitBuilder::new()
        .client(reqwest::Client::new())
        .endpoint("http://localhost:8080")
        .credential(Credential::new("admin", "adminadmin"))
        .build();

    QbitBuilder::new()
        .endpoint("http://localhost:8080")
        .credential(Credential::new("admin", "adminadmin"))
        .build();

    QbitBuilder::new()
        .client(reqwest::Client::new())
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .build();

    QbitBuilder::new()
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .build();

    QbitBuilder::new()
        .basic_auth_credentials(Some(Credential::new("basic", "auth")))
        .client(reqwest::Client::new())
        .endpoint("http://localhost:8080")
        .credential(Credential::new("admin", "adminadmin"))
        .build();

    QbitBuilder::new()
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .basic_auth_credentials(None)
        .build();
}

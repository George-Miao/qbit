#![allow(private_interfaces, private_bounds)]

use std::{fmt::Debug, sync::Mutex};

use tap::Pipe;
use url::Url;

use crate::{ext::Cookie, model::Credential, LoginState, Qbit};

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
    pub fn client<Cl>(self, client: Cl) -> QbitBuilder<C, Cl, E> {
        QbitBuilder {
            credential: self.credential,
            client,
            endpoint: self.endpoint,
        }
    }

    #[allow(private_interfaces)]
    pub fn cookie(self, cookie: impl Into<String>) -> QbitBuilder<Cookie, R, E> {
        QbitBuilder {
            credential: Cookie(cookie.into()),
            client: self.client,
            endpoint: self.endpoint,
        }
    }

    pub fn credential(self, credential: Credential) -> QbitBuilder<Credential, R, E> {
        QbitBuilder {
            credential,
            client: self.client,
            endpoint: self.endpoint,
        }
    }

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

impl<C, U> QbitBuilder<C, reqwest::Client, U>
where
    C: IntoLoginState,
    U: TryInto<Url>,
    U::Error: Debug,
{
    pub fn build(self) -> Qbit {
        let endpoint = self.endpoint.try_into().expect("Invalid endpoint");
        let client = reqwest::Client::new();
        let state = self.credential.into_login_state().pipe(Mutex::new);

        Qbit {
            endpoint,
            client,
            state,
        }
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
        .client(reqwest::Client::new())
        .endpoint("http://localhost:8080")
        .cookie("SID=1234567890")
        .build();
}

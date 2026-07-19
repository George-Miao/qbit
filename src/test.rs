use std::{
    env,
    ops::Deref,
    sync::{LazyLock, Once, OnceLock},
};

use tracing::info;

use super::*;
use crate::model::*;

#[cfg(feature = "reqwest")]
async fn sleep(duration: std::time::Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(feature = "cyper")]
async fn sleep(duration: std::time::Duration) {
    compio::time::sleep(duration).await;
}

async fn init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        dotenv::dotenv().expect("Failed to load .env file");
        tracing_subscriber::fmt::init();
    });
}

async fn client_with_credentials<'a>() -> Result<&'a Qbit> {
    init().await;
    static PREPARE: LazyLock<(Credential, Url)> = LazyLock::new(|| {
        (
            Credential::new(
                env::var("QBIT_USERNAME").expect("QBIT_USERNAME not set"),
                env::var("QBIT_PASSWORD").expect("QBIT_PASSWORD not set"),
            ),
            env::var("QBIT_BASEURL")
                .expect("QBIT_BASEURL not set")
                .parse()
                .expect("QBIT_BASEURL is not a valid url"),
        )
    });
    static API: OnceLock<Qbit> = OnceLock::new();

    if let Some(api) = API.get() {
        Ok(api)
    } else {
        let (credential, url) = PREPARE.deref().clone();
        let api = Qbit::new(url, credential);
        api.login(false).await?;
        drop(API.set(api));
        Ok(API.get().unwrap())
    }
}

async fn client_with_api_key<'a>() -> Result<&'a Qbit> {
    init().await;
    static PREPARE: LazyLock<Option<(String, Url)>> = LazyLock::new(|| {
        let api_key = env::var("QBIT_API_KEY").ok()?;
        let url = env::var("QBIT_BASEURL")
            .expect("QBIT_BASEURL not set")
            .parse()
            .expect("QBIT_BASEURL is not a valid url");
        Some((api_key, url))
    });
    static API: OnceLock<Option<Qbit>> = OnceLock::new();

    let prepared = PREPARE.deref();
    let Some((api_key, url)) = prepared.clone() else {
        return Err(Error::ApiError(ApiError::NotLoggedIn));
    };

    if let Some(Some(api)) = API.get() {
        Ok(api)
    } else {
        let api = Qbit::builder().endpoint(url).api_key(api_key).build();
        drop(API.set(Some(api)));
        Ok(API.get().unwrap().as_ref().unwrap())
    }
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_login() {
    let client = client_with_credentials().await.unwrap();

    info!(
        version = client.get_version().await.unwrap(),
        "Login success"
    );
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_login_bad_credentials() {
    init().await;
    let url: Url = env::var("QBIT_BASEURL")
        .expect("QBIT_BASEURL not set")
        .parse()
        .expect("QBIT_BASEURL is not a valid url");
    let client = Qbit::new(url, Credential::new("no_such_user", "wrong_password"));
    let err = client.login(true).await.unwrap_err();
    assert!(matches!(err, Error::ApiError(ApiError::BadCredentials)));
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_version_api_key() {
    let client = match client_with_api_key().await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("QBIT_API_KEY not set, skipping API key test");
            return;
        }
    };

    info!(
        version = client.get_version().await.unwrap(),
        "Login success"
    );
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_preference() {
    let client = client_with_credentials().await.unwrap();

    client.get_preferences().await.unwrap();
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_add_torrent() {
    let client = client_with_credentials().await.unwrap();
    let arg = AddTorrentArg {
        source: TorrentSource::Urls {
            urls: vec![
                "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.torrent"
                    .parse()
                    .unwrap(),
            ]
            .into(),
        },
        ratio_limit: Some(1.0),
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
}
#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_add_torrent_file() {
    let client = client_with_credentials().await.unwrap();
    let arg = AddTorrentArg {
        source: TorrentSource::TorrentFiles {
            torrents: vec![ TorrentFile {
                filename: "leaves.torrent".into(),
                data: client::get("https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/leaves.torrent")
                    .await
                    .unwrap()
                    .bytes()
                    .await
                    .unwrap()
                    .to_vec(),
            }]
        },
        ratio_limit: Some(1.0),
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_get_torrent_list() {
    let client = client_with_credentials().await.unwrap();
    let list = client
        .get_torrent_list(GetTorrentListArg::default())
        .await
        .unwrap();
    print!("{:#?}", list);
}

#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_download_torrent_file() {
    let client = client_with_credentials().await.unwrap();
    let expected = client::get(
        "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.txt",
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();
    let arg = AddTorrentArg {
        source: TorrentSource::Urls {
            urls: vec![
                "https://github.com/webtorrent/webtorrent-fixtures/raw/d20eec0ae19a18b088cf7b221ff70bb9f840c226/fixtures/alice.torrent"
                    .parse()
                    .unwrap(),
            ]
            .into(),
        },
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
    let mut hash = None;
    for _ in 0..30 {
        let list = client
            .get_torrent_list(GetTorrentListArg::default())
            .await
            .unwrap();
        hash = list
            .iter()
            .find(|torrent| torrent.name.as_deref() == Some("alice.txt"))
            .and_then(|torrent| torrent.hash.clone());
        if hash.is_some() {
            break;
        }
        sleep(std::time::Duration::from_secs(1)).await;
    }
    let hash = hash.expect("alice torrent was not added in time");

    // Wait for the torrent to finish downloading.
    let mut completed = false;
    for _ in 0..30 {
        let props = client.get_torrent_properties(&hash).await.unwrap();
        if props.completion_date.is_some_and(|date| date >= 0) {
            completed = true;
            break;
        }
        sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(completed, "alice torrent did not complete in time");

    let data = client.download_torrent_file(&hash, "0").await.unwrap();
    let content = String::from_utf8(data.to_vec()).unwrap();
    assert_eq!(content, expected);
}

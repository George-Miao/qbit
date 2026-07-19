use std::{
    env,
    ops::Deref,
    sync::{LazyLock, Once},
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

async fn client_with_credentials() -> Result<Qbit> {
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
    let (credential, url) = PREPARE.deref().clone();
    let api = Qbit::new(url, credential);
    api.login(false).await?;
    Ok(api)
}

async fn client_with_api_key() -> Result<Qbit> {
    init().await;
    static PREPARE: LazyLock<Option<(String, Url)>> = LazyLock::new(|| {
        let api_key = env::var("QBIT_API_KEY").ok()?;
        let url = env::var("QBIT_BASEURL")
            .expect("QBIT_BASEURL not set")
            .parse()
            .expect("QBIT_BASEURL is not a valid url");
        Some((api_key, url))
    });
    let prepared = PREPARE.deref();
    let Some((api_key, url)) = prepared.clone() else {
        return Err(Error::ApiError(ApiError::NotLoggedIn));
    };

    Ok(Qbit::builder().endpoint(url).api_key(api_key).build())
}

async fn remove_torrent_if_present(client: &Qbit, name: &str) {
    let hashes = client
        .get_torrent_list(GetTorrentListArg::default())
        .await
        .unwrap()
        .into_iter()
        .filter(|torrent| torrent.name.as_deref() == Some(name))
        .filter_map(|torrent| torrent.hash)
        .collect::<Vec<_>>();

    if !hashes.is_empty() {
        client.delete_torrents(hashes, false).await.unwrap();
    }
}

async fn wait_for_torrent(client: &Qbit, name: &str) -> String {
    for _ in 0..50 {
        let hash = client
            .get_torrent_list(GetTorrentListArg::default())
            .await
            .unwrap()
            .into_iter()
            .find(|torrent| torrent.name.as_deref() == Some(name))
            .and_then(|torrent| torrent.hash);
        if let Some(hash) = hash {
            return hash;
        }
        sleep(std::time::Duration::from_millis(100)).await;
    }

    panic!("{name} torrent was not added in time");
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
    remove_torrent_if_present(&client, "numbers").await;
    let fixture_baseurl =
        env::var("QBIT_FIXTURE_BASEURL").unwrap_or_else(|_| "http://127.0.0.1:18080".into());
    let arg = AddTorrentArg {
        source: TorrentSource::Urls {
            urls: vec![
                format!("{fixture_baseurl}/numbers.torrent")
                    .parse()
                    .unwrap(),
            ]
            .into(),
        },
        ratio_limit: Some(1.0),
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
    let hash = wait_for_torrent(&client, "numbers").await;
    client.delete_torrents(vec![hash], false).await.unwrap();
}
#[cfg_attr(feature = "reqwest", tokio::test)]
#[cfg_attr(feature = "cyper", compio::test)]
async fn test_add_torrent_file() {
    let client = client_with_credentials().await.unwrap();
    remove_torrent_if_present(&client, "Leaves of Grass by Walt Whitman.epub").await;
    let arg = AddTorrentArg {
        source: TorrentSource::TorrentFiles {
            torrents: vec![TorrentFile {
                filename: "leaves.torrent".into(),
                data: include_bytes!("../tests/fixtures/leaves.torrent").to_vec(),
            }],
        },
        ratio_limit: Some(1.0),
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
    let hash = wait_for_torrent(&client, "Leaves of Grass by Walt Whitman.epub").await;
    client.delete_torrents(vec![hash], false).await.unwrap();
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
    let expected = include_str!("../tests/fixtures/alice.txt");
    remove_torrent_if_present(&client, "alice.txt").await;

    let arg = AddTorrentArg {
        source: TorrentSource::TorrentFiles {
            torrents: vec![TorrentFile {
                filename: "alice.torrent".into(),
                data: include_bytes!("../tests/fixtures/alice.torrent").to_vec(),
            }],
        },
        savepath: Some(format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))),
        ..AddTorrentArg::default()
    };
    client.add_torrent(arg).await.unwrap();
    let hash = wait_for_torrent(&client, "alice.txt").await;

    // qBittorrent hash-checks the content already present in the fixture directory.
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
    client.delete_torrents(vec![hash], false).await.unwrap();
    assert_eq!(content, expected);
}

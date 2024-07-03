# Qbit-rs

[<img alt="crates.io" src="https://img.shields.io/crates/v/qbit-rs?style=for-the-badge&labelColor=555555&color=FFD3B6&logo=rust" height="20">](https://crates.io/crates/qbit-rs)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-qbit--rs-DCEDC1?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/qbit-rs)
[<img alt="github" src="https://img.shields.io/badge/gitub-George--Miao-A8E6CF?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/George-Miao/qbit)

A Rust library for interacting with qBittorrent's Web API.

Implemented according to [WebUI API (qBittorrent 4.1)](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)).

## Usage

Add dependency by running:

```bash
cargo add qbit-rs
```

or manually add to `Cargo.toml`:

```toml
[dependencies]
qbit-rs = "0.4"
```

Then use it in your code:

```rust,ignore
use qbit_rs::Qbit;
use qbit_rs::model::Credential;

let credential = Credential::new("username", "password");
let api = Qbit::new("http://my-qb-instance.domain", credential);
let torrents = api.get_version().await;
```

or use the builder pattern:

```rust,ignore
use qbit_rs::Qbit;

let api = Qbit::builder()
    .endpoint("http://my-qb-instance.domain")
    .cookie("SID=1234567890")
    .build();
let torrents = api.get_version().await;
```

For more methods, see [`Qbit`](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html).

## API Coverage

Most of the API is covered, except `RSS` and `Search`. PR is welcomed if you need that part of the API. The following is a list of the implementation status:

1. [x] Authentication
   1. [x] [Login](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.login)
   1. [x] [Logout](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.logout)
1. [x] Application
   1. [x] [Get application version](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_version)
   1. [x] [Get API version](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_webapi_version)
   1. [x] [Get build info](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_build_info)
   1. [x] [Shutdown application](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.shutdown)
   1. [x] [Get application preferences](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_preferences)
   1. [x] [Set application preferences](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_preferences)
   1. [x] [Get default save path](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_default_save_path)
1. [x] Log
   1. [x] [Get log](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_logs)
   1. [x] [Get peer log](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_peer_logs)
1. [x] Sync
   1. [x] [Get main data](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.sync)
   1. [x] [Get torrent peers data](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_peers)
1. [x] Transfer info
   1. [x] [Get global transfer info](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_transfer_info)
   1. [x] [Get alternative speed limits state](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_speed_limits_mode)
   1. [x] [Toggle alternative speed limits](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.toggle_speed_limits_mode)
   1. [x] [Get global download limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_download_limit)
   1. [x] [Set global download limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_download_limit)
   1. [x] [Get global upload limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_upload_limit)
   1. [x] [Set global upload limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_upload_limit)
   1. [x] [Ban peers](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.ban_peers)
1. [x] Torrent management
   1. [x] [Get torrent list](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_list)
   1. [x] [Get torrent generic properties](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_properties)
   1. [x] [Get torrent trackers](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_trackers)
   1. [x] [Get torrent web seeds](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_web_seeds)
   1. [x] [Get torrent contents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_contents)
   1. [x] [Get torrent pieces' states](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_pieces_stats)
   1. [x] [Get torrent pieces' hashes](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_pieces_hashes)
   1. [x] [Add new torrent](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_torrent)
   1. [x] [Pause torrents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.pauce_torrents)
   1. [x] [Resume torrents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.resume_torrents)
   1. [x] [Delete torrents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.delete_torrents)
   1. [x] [Recheck torrents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.recheck_torrents)
   1. [x] [Reannounce torrents](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.reannounce_torrents)
   1. [x] [Edit trackers](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.edit_trackers)
   1. [x] [Remove trackers](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.remove_trackers)
   1. [x] [Add peers](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_peers)
   1. [x] [Add trackers to torrent](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_trackers)
   1. [x] [Increase torrent priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.increase_priority)
   1. [x] [Decrease torrent priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.decrease_priority)
   1. [x] [Maximal torrent priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.maximal_priority)
   1. [x] [Minimal torrent priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.minimal_priority)
   1. [x] [Set file priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_file_priority)
   1. [x] [Get torrent download limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_download_limit)
   1. [x] [Set torrent download limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_download_limit)
   1. [x] [Set torrent share limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_shared_limit)
   1. [x] [Get torrent upload limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_torrent_upload_limit)
   1. [x] [Set torrent upload limit](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_upload_limit)
   1. [x] [Set torrent location](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_location)
   1. [x] [Set torrent name](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_name)
   1. [x] [Set torrent category](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_torrent_category)
   1. [x] [Get all categories](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_categories)
   1. [x] [Add new category](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_category)
   1. [x] [Edit category](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.edit_categories)
   1. [x] [Remove categories](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.remove_categories)
   1. [x] [Add torrent tags](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_torrent_tags)
   1. [x] [Remove torrent tags](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.remove_torrent_tags)
   1. [x] [Get all tags](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.get_all_tags)
   1. [x] [Create tags](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.create_tags)
   1. [x] [Delete tags](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.delete_tags)
   1. [x] [Set automatic torrent management](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_auto_management)
   1. [x] [Toggle sequential download](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.toggle_sequential_download)
   1. [x] [Set first/last piece priority](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.toggle_first_last_piece_priority)
   1. [x] [Set force start](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_force_star)
   1. [x] [Set super seeding](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.set_super_seeding)
   1. [x] [Rename file](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.rename_file)
   1. [x] [Rename folder](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.rename_folder)
1. [ ] RSS (experimental)
   1. [x] [Add folder](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_folder)
   1. [x] [Add feed](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.add_feed)
   1. [x] [Remove item](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.remove_item)
   1. [x] [Move item](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.move_item)
   1. [ ] Get all items
   1. [x] [Mark as read](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.mark_as_read)
   1. [x] [Refresh item](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.refresh_item)
   1. [ ] Set auto-downloading rule
   1. [x] [Rename auto-downloading rule](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.rename_rule)
   1. [x] [Remove auto-downloading rule](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.remove_rule)
   1. [ ] Get all auto-downloading rules
   1. [ ] Get all articles matching a rule
1. [ ] Search
   1. [ ] Start search
   1. [ ] Stop search
   1. [ ] Get search status
   1. [ ] Get search results
   1. [ ] Delete search
   1. [ ] Get search plugins
   1. [ ] Install search plugin
   1. [ ] Uninstall search plugin
   1. [ ] Enable search plugin
   1. [ ] Update search plugins
1. Undocumented
   1. [x] [Export torrent](https://docs.rs/qbit-rs/latest/qbit_rs/struct.Qbit.html#method.export_torrent)[^1]

[^1]: The endpoint is added in [this PR](https://github.com/qbittorrent/qBittorrent/pull/16968)
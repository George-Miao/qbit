# AGENTS.md

## Project

`qbit-rs` is a Rust 2021 client for qBittorrent's Web API. It supports the
default `reqwest` backend and the optional `cyper` backend. Public behavior and
terminology should follow qBittorrent's official WebUI API documentation.

## Layout

- `src/lib.rs`: crate entry point, `Qbit`, shared request flow, and error types.
- `src/endpoint/`: API methods grouped by qBittorrent endpoint family. Each
  file adds methods in an `impl Qbit` block.
- `src/model/`: request and response types grouped by the same API families.
- `src/client.rs`: backend abstraction for `reqwest` and `cyper`.
- `src/builder.rs`: `QbitBuilder` and client construction.
- `src/test.rs`: API tests; group related, non-interfering methods in one test.
- `tests/fixtures/`: torrent, RSS, and search-plugin fixtures used by API tests.
- `README.md`: usage, authentication, and endpoint coverage.

## Style

- Follow existing module boundaries and naming; add endpoints to the matching
  `src/endpoint/<family>.rs` file and models to `src/model/<family>.rs`.
- Keep public API documentation complete. The crate denies `missing_docs` and
  `rustdoc::broken_intra_doc_links`.
- Base endpoint documentation on qBittorrent's official descriptions and link
  to the corresponding official section.
- Put documentation comments before all attributes, including `derive`,
  `serde`, and `cfg_attr`.
- For fallible public methods, include `# Errors`. List each returned
  `ApiError` variant and the scenario that produces it; describe it through the
  crate's `Error` wrapper rather than exposing backend HTTP errors.
- Prefer typed request structs and Serde attributes over manual form or JSON
  construction.
- Keep changes scoped, use ASCII unless the surrounding file requires Unicode,
  and run `rustfmt` on edited Rust files.

## Verification

Run these before submitting changes:

```sh
cargo +nightly clippy --all-targets -- -Dwarnings
cargo +nightly clippy --all-targets --no-default-features --features cyper -- -Dwarnings
cargo +nightly doc --workspace --no-deps
```

The full `cargo +nightly test` suite talks to a real qBittorrent instance and
uses `.env` plus `tests/fixtures/`; see `.github/workflows/test.yml` for the
Docker setup and required environment variables.

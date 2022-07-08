![Build](https://github.com/nknotts/rust-key-value-store-demo/actions/workflows/rust.yml/badge.svg)

# Rust `kv_store_demo` Application

Inspired by [Introduction to Rust Part 1](https://www.youtube.com/watch?v=WnWGO-tLtLA) and [Dynamic vs Static Dispatch in Rust](https://www.youtube.com/watch?v=tM2r9HD4ivQ) by [Ryan Levick](https://www.youtube.com/c/RyanLevicksVideos).

I wanted to write a hello world style application to get my feet wet with [Rust](https://www.rust-lang.org/).

Features:
 * Cli argument parsing via [clap](https://docs.rs/clap/3.2.8/clap/)
 * YAML serialization via [serde_yaml](https://docs.rs/serde_yaml/0.8.24/serde_yaml/)
 * JSON serialization via [serde_json](https://docs.rs/serde_json/1.0.82/serde_json/)
 * CSV serialization via [csv](https://docs.rs/csv/1.1.6/csv/)
 * SQLite serialization via [rusqlite](https://docs.rs/rusqlite/0.27.0/rusqlite/)

Supported CLI Commands
 * `init`: create an empty key/value store
 * `list`: list the contents of the key/value store
 * `add`: add a key/value pair to the store
 * `remove`: remove a key/value pair from the store

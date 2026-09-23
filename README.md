# Aegisub Converter

A small Rust command-line utility that expands Aegisub Extradata placeholders in an `.ass` subtitle file and removes the `[Aegisub Extradata]` section from the converted copy.

## Usage

```sh
cargo run --release -- path/to/subtitle.ass
```

The converted file is written next to the input as `subtitle_converted.ass`. The original file is left unchanged. On Windows, the executable can also be opened by dropping an `.ass` file onto it.

## Build and test

```sh
cargo test
cargo build --release
```

GitHub Actions checks formatting, runs Clippy and tests, and builds on Ubuntu and macOS for pushes to `main`, pull requests, and manual runs.

# ggwave-core

Platform-neutral Rust building blocks for ggwave.

It provides codec encode/decode, stable protocol IDs, configurable ultrasonic tuning, optional 1.8x pre-emphasis, and optional 800 ms packet deduplication. It does **not** depend on Flutter, microphone/speaker APIs, QR, or any application protocol.

## Features

- `ultrasonic` (default) — ultrasonic frequency tuning and pre-emphasis.
- `dedup` (default) — sliding-window packet deduplication.
- `--no-default-features` — minimal codec wrapper.

## Platforms

The core has no audio-device dependency. Primary native targets are Android, iOS, macOS, Windows and Linux. Web/WASM is tracked separately because browser audio capture/playback must use Web Audio rather than native `cpal`.

See `docs/PLATFORMS.md` in the repository.

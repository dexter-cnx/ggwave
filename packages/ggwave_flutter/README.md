# ggwave_rs_flutter

Rust-backed Flutter transport for sending and receiving small data payloads over audible or ultrasonic sound.

This package is intentionally application-agnostic. It does not contain Bingo, QR, pairing, device identity, or other application protocols.

## Architecture

`ggwave_rs_flutter` -> Flutter/Rust bridge -> native audio adapter -> `ggwave-core`

The native adapter owns microphone/speaker streaming. `ggwave-core` owns codec, protocol mapping, ultrasonic tuning and packet deduplication.

## Platforms

Planned Tier 1 native targets:

- Android
- iOS
- macOS
- Windows
- Linux

Web is planned separately using Web Audio plus WASM/JS rather than the CPAL native backend.

## Usage

```dart
import 'package:ggwave_rs_flutter/ggwave_rs_flutter.dart';

final transport = GgWaveFlutterTransport();
await transport.initialize();
await transport.startListening();

transport.messages.listen((payload) {
  // Application-specific protocol handling lives here.
});
```

## Development

Before the first publish, generate and commit the native scaffold and FRB bindings:

```bash
./tool/bootstrap_flutter_native.sh
./tool/release_check.sh
```

The pub.dev name `ggwave_flutter` is already owned by another project, so this implementation publishes as `ggwave_rs_flutter`.

# Tic-Tac-Toe acoustic reference demo

The Flutter Tic-Tac-Toe example demonstrates a small interactive 1 ↔ 1 application protocol over the public `ggwave_rs_flutter` transport.

This is intentionally different from application-level broadcast consumers such as Bingo. Tic-Tac-Toe exercises bidirectional peer interaction; broadcast applications exercise one sender to many listeners.

## Architecture boundary

```text
Tic-Tac-Toe game state + wire format
                ↓
        ggwave_rs_flutter
                ↓
            ggwave_dart
                ↓
          FRB / Rust core
                ↓
        speaker ↔ microphone
```

The game protocol stays under `packages/ggwave_flutter/example/`. No Tic-Tac-Toe message type, board state, session ID, or turn rule is added to `ggwave-core` or the publishable Dart/Flutter API.

## Wire format

The demo uses a compact UTF-8 packet because the payload is already tiny and human-readable diagnostics are useful during hardware validation:

```text
TT1|TYPE|SESSION|SENDER|SEQUENCE|BOARD
```

Message types:

```text
H  host/session announcement
R  joiner ready
S  board-state update
X  round reset
```

Example:

```text
TT1|S|A1B2C3|D4E5F6|7|X-O-X----
```

This remains far below the repository's 140-byte payload limit.

## Reliability behavior

The demo deliberately keeps reliability at the application edge so the universal codec stays payload-agnostic:

- each peer has a short sender ID;
- every packet carries a monotonically increasing sender-local sequence;
- duplicate or out-of-order packets from the same sender are ignored;
- moves send the complete 9-cell board snapshot instead of only a cell index;
- a remote snapshot is accepted only when it changes exactly one empty cell to the remote player's mark;
- self-heard packets are ignored by sender ID;
- session IDs keep nearby independent games separated.

Sending a full board is inexpensive for Tic-Tac-Toe and makes the demo easier to recover and inspect than a move-only stream.

## Run from VSCode

1. Connect two physical Android devices.
2. On each device choose **ggwave Tic-Tac-Toe Demo** from **Run and Debug**.
3. Press **F5**.
4. Device A taps **Host as X**.
5. Device B taps **Join as O**.
6. Keep the devices close enough for reliable audible playback/capture and play normally.

The launch target reuses the same Android/FRB preparation task as the existing hardware-validation app, so no CI APK copying is required.

## CI

Flutter Android CI formats and tests the example protocol and builds both Android entrypoints:

```text
ggwave-validation-debug.apk
ggwave-tic-tac-toe-debug.apk
```

Acoustic behavior still requires two physical devices; CI validates compilation, static analysis, and deterministic protocol logic only.

# Code Walkthrough — ggwave

เอกสารนี้อธิบายโครงสร้างและ data flow ของ repository `ggwave` ตั้งแต่ Rust core ไปจนถึง Flutter และ native Android/Kotlin โดยยึดหลักว่า core ต้องเป็น universal และไม่ผูกกับ application protocol เช่น Bingo, QR, pairing หรือ game state

## 1. Architecture overview

```text
                           Application layer
                    ┌─────────────┴─────────────┐
                    │                           │
              Flutter / Dart              Android / Kotlin
                    │                           │
          ggwave_rs_flutter                ggwave-kotlin
                    │                           │
                   FRB                         JNI
                    │                           │
                    │                    ggwave-jni worker
                    │                           │
                    └──────────┬────────────────┘
                               │
                         ggwave-core
                               │
                           ggwave-rs
```

`ggwave-core` ไม่รู้จัก Flutter, Android, microphone, speaker หรือ payload semantics ของ application ใด ๆ การแบ่ง layer แบบนี้ทำให้ Rust core สามารถ reuse กับ binding ภาษาอื่นได้โดยไม่ต้องลาก Flutter/Android dependency เข้าไป

## 2. Rust core — `crates/ggwave-core`

ไฟล์หลัก:

```text
crates/ggwave-core/src/lib.rs
```

core เป็น source of truth ของ:

- protocol IDs
- payload size limit
- codec initialization
- sample rate
- ultrasonic tuning
- ultrasonic pre-emphasis
- encode/decode
- optional packet deduplication

### 2.1 Stable protocol IDs

```rust
pub enum Protocol {
    AudibleFast = 1,
    UltrasonicFast = 5,
}
```

`UltrasonicFast = 5` เป็น stable app-facing ID ของ repository นี้ ส่วนภายใน map ไป upstream `GGWAVE_PROTOCOL_ULTRASOUND_FAST` ดังนั้น consumer ไม่ต้องรู้ enum value ภายในของ upstream library

### 2.2 Core defaults

```text
MAX_PAYLOAD               140 bytes
sample rate               48,000 Hz
ultrasonic start          12,000 Hz
ultrasonic pre-emphasis   1.8
dedup window              800 ms
frequency range           8,000..19,000 Hz
```

`Tuning::apply()` ตั้งทั้ง TX และ RX ultrasonic start frequency ผ่าน ggwave FFI เพื่อให้ encode/decode ใช้ profile เดียวกัน

### 2.3 Codec ownership

```rust
pub struct Codec {
    inner: GgWave,
    tuning: Tuning,
}
```

upstream `GgWave` เป็น `!Send` / `!Sync` ดังนั้น binding ห้ามแชร์ instance ข้าม thread แบบทั่วไป นี่เป็นเหตุผลสำคัญที่ JNI layer ใช้ dedicated worker thread แทนการเขียน `unsafe impl Send`

### 2.4 Encode path

```text
payload bytes
   ↓
validate <= 140 bytes
   ↓
validate volume 0..100
   ↓
apply tuning
   ↓
ggwave encode
   ↓
F32 PCM bytes
   ↓
Vec<f32>
   ↓
optional ultrasonic pre-emphasis
```

ultrasonic pre-emphasis ปัจจุบันใช้:

```rust
(input - 0.85 * previous) * gain
```

แล้ว clamp เป็น `[-1.0, 1.0]`

### 2.5 Decode path

```text
Float32 mono PCM
   ↓
f32_to_bytes
   ↓
ggwave decode
   ↓
Option<Vec<u8>>
```

`None` หมายถึง decoder ยังไม่มี complete packet จาก chunk ปัจจุบัน

### 2.6 Dedup

เมื่อเปิด feature `dedup`, `PacketDeduper` suppress payload ซ้ำภายใน sliding window 800 ms โดยไม่ตีความ sequence number หรือ application-specific semantics

## 3. JNI bridge — `crates/ggwave-jni`

ไฟล์หลัก:

```text
crates/ggwave-jni/src/lib.rs
```

JNI ต้องรองรับการถูกเรียกจากหลาย JVM threads แต่ codec upstream ไม่ควรถูกย้ายข้าม thread จึงใช้ architecture:

```text
Kotlin/Java caller
      ↓
JNI function
      ↓
mpsc Command
      ↓
Dedicated Rust worker
      ↓
single owned Codec instance
```

worker commands ปัจจุบัน:

```text
SetFrequency
Encode
Decode
```

แต่ละ command มี reply channel กลับ caller

ข้อดี:

- codec ถูกสร้าง/ใช้งานบน Rust worker เดียว
- Kotlin เรียกจาก main/background threads ได้
- ไม่มี `unsafe impl Send for GgWave`
- thread ownership audit ได้ง่าย

JNI exports:

```text
nativeSetUltrasonicFrequency
nativeEncode
nativeDecode
```

Rust errors ถูกส่งกลับเป็น `IllegalStateException`

## 4. Kotlin Android binding — `packages/ggwave_kotlin`

package นี้เป็น native Android AAR และไม่ต้องมี Flutter

```text
minSdk       23
compileSdk   36
AGP          9.0.0
Gradle       >= 9.1
Kotlin       built-in Kotlin
ABIs         arm64-v8a, armeabi-v7a, x86_64
```

เปิด built-in Kotlin ผ่าน:

```properties
android.builtInKotlin=true
```

และไม่ apply legacy `org.jetbrains.kotlin.android` plugin

### 4.1 Low-level codec API

```kotlin
GgWave.setUltrasonicFrequency(12_000f)

val waveform = GgWave.encode(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)

val payload = GgWave.decode(samples)
```

เหมาะกับ app ที่มี audio engine ของตัวเอง

### 4.2 High-level Android audio API

```kotlin
GgWaveAudio.startListening { payload ->
    println(payload.decodeToString())
}

GgWaveAudio.send(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)
```

capture ใช้ `AudioRecord`, playback ใช้ `AudioTrack`, PCM เป็น 48 kHz mono float ให้ตรงกับ codec default

`MessageListener` เป็น Kotlin `fun interface` จึงใช้ lambda ได้และ Java caller ใช้เป็น SAM interface ได้

### 4.3 Permission ownership

AAR declare `android.permission.RECORD_AUDIO` แต่ runtime permission request เป็นหน้าที่ host app เพราะ library ไม่ควรผูก UI/Activity policy ให้ consumer

receive callback ทำงานบน capture thread ถ้าจะอัปเดต UI ต้อง dispatch กลับ main thread

## 5. Kotlin hardware validation app

อยู่ที่:

```text
examples/ggwave_kotlin_validation
```

app นี้ consume public Kotlin API จริง ไม่ bypass JNI เพื่อให้ compile/run behavior ใกล้ consumer จริงที่สุด

รองรับ:

- request microphone permission
- start/stop listening
- Audible Fast
- Ultrasonic 12 kHz
- Ultrasonic 15 kHz
- Ultrasonic 18 kHz
- sent/received counters
- last received payload
- stop listening on lifecycle pause

CI compile app นี้ทุกครั้งเพื่อกัน public API drift

## 6. Kotlin Android validated build path

GitHub Actions workflow `Kotlin Android` ตรวจ:

```text
cargo fmt --check
      ↓
cargo check -p ggwave-jni
      ↓
cargo clippy -p ggwave-jni -- -D warnings
      ↓
cargo-ndk
      ├─ arm64-v8a
      ├─ armeabi-v7a
      └─ x86_64
      ↓
AGP 9 + built-in Kotlin + Gradle 9.1
      ↓
release AAR
      ↓
Maven POM
      ↓
compile standalone validation app
      ↓
upload AAR + validation APK
```

validated evidence ล่าสุด:

```text
workflow: Kotlin Android #18
run id:   33600474569
status:   success
```

artifacts:

```text
ggwave-kotlin-release
ggwave-kotlin-validation-apk
```

artifact digests:

```text
AAR SHA-256: 0b34957c62ac752d1856584bf543007b53d59e981bfc4f1e3c5c2721d2c3606f
APK SHA-256: 6ca205abe5c17150be07e1b59d1c6f23b41b96b9adb45f58728c7750cb737be1
```

นี่คือ **build validation** ยังไม่ใช่ acoustic hardware validation

## 7. Dart package — `packages/ggwave_dart`

pure Dart layer ไม่มี Flutter/FFI dependency

หน้าที่:

- protocol model
- tuning model
- sequence deduper
- transport abstraction

application logic จึงสามารถ depend กับ Dart contract โดยไม่ต้องรู้ native implementation และ package นี้ไม่รู้จัก Bingo packet หรือ QR format

## 8. Flutter package — `packages/ggwave_flutter`

publish name:

```text
ggwave_rs_flutter
```

ไม่ใช้ชื่อ `ggwave_flutter` เพราะมี third-party package ใช้ชื่อนั้นบน pub.dev อยู่แล้ว

baseline:

```text
Flutter >= 3.47.0
Dart    >= 3.12.0
FRB      2.8.0
```

Flutter 3.47 CI ถูก pin ที่ `3.47.0` โดยตรงเพื่อให้ compatibility reproducible

### 8.1 Responsibility

Flutter layer รับผิดชอบ:

- Flutter-facing transport API
- FRB boundary
- native audio lifecycle
- microphone/speaker I/O
- platform packaging

codec/tuning logic ต้องมาจาก `ggwave-core` ไม่ duplicate ที่ Flutter layer

### 8.2 Monorepo development dependency

ก่อน `ggwave_dart` publish, `packages/ggwave_flutter/pubspec_overrides.yaml` ชี้ `ggwave_dart` ไปที่ local sibling package เพื่อให้ CI/test monorepo resolve ได้โดยไม่ต้องปล่อย package ก่อนเวลา

### 8.3 Flutter Android CI

workflow `Flutter Android` ใช้:

```text
Flutter 3.47.0 stable
FRB codegen 2.8.0
Android SDK 36
NDK 27
Rust stable
```

flow ที่ตั้งใจ validate:

```text
pub get Dart/Flutter/example
      ↓
FRB integrate Cargokit for Android
      ↓
FRB generate
      ↓
dart format/analyze/test
      ↓
flutter format/analyze/test
      ↓
build Android example APK
      ↓
upload example APK
```

จนกว่า workflow นี้เขียว Flutter Android ยังถือว่า implementation present / build validation pending

## 9. End-to-end Kotlin send flow

```text
Application payload
   ↓
GgWaveAudio.send()
   ↓
GgWave.encode()
   ↓ JNI
Command::Encode
   ↓
Rust worker
   ↓
ggwave_core::Codec::encode
   ↓
ggwave-rs
   ↓
Float32 PCM waveform
   ↓ JNI
Kotlin FloatArray
   ↓
AudioTrack
   ↓
Speaker
```

## 10. End-to-end Kotlin receive flow

```text
Microphone
   ↓
AudioRecord @ 48 kHz mono
   ↓
FloatArray chunk
   ↓
GgWave.decode()
   ↓ JNI
Command::Decode
   ↓
Rust worker
   ↓
ggwave_core::Codec::decode
   ↓
complete payload?
   ├─ no  -> continue listening
   └─ yes -> MessageListener
```

## 11. Flutter conceptual flow

Send:

```text
Dart payload
   ↓
ggwave_rs_flutter
   ↓ FRB
Rust native adapter
   ↓
ggwave-core
   ↓
PCM
   ↓
native audio output
```

Receive เป็นเส้นทางกลับกันจาก native audio input → Rust decoder → FRB stream → Dart transport

## 12. Platform plan

Tier 1 Flutter:

```text
Android
iOS
macOS
Windows
Linux
```

Web เป็น Tier 2 และควรใช้ Web Audio + AudioWorklet + WASM/JS backend แยก เพราะ browser permission/audio lifecycle/sample-rate behavior ต่างจาก CPAL native path

## 13. Hardware validation ที่ยัง pending

Android physical-device acceptance ต้องทดสอบอย่างน้อย:

- microphone runtime permission
- audible roundtrip
- ultrasonic 12 kHz roundtrip
- ultrasonic 15 kHz roundtrip
- ultrasonic 18 kHz roundtrip
- two-device TX/RX สลับทิศทาง
- distance/orientation matrix
- background/resume
- 10 start/listen/stop cycles
- release-mode consumer behavior

รายละเอียดอยู่ใน `docs/ANDROID_VALIDATION.md`

## 14. Release topology

```text
1. ggwave-core       -> crates.io
2. ggwave_dart       -> pub.dev
3. ggwave_rs_flutter -> pub.dev
4. ggwave-kotlin     -> Maven repository
```

`ggwave-jni` เป็น support crate สำหรับ Android binding และไม่จำเป็นต้องเป็น user-facing artifact แยก

## 15. Application boundary

สิ่งเหล่านี้ไม่ควรอยู่ใน repo `ggwave`:

```text
BINGO:JOIN:<gid>
BINGO:NUM:<gid>:<num>:<seq>
player ID
game state
QR fallback policy
pairing token format
application retry semantics
```

`ggwave` ควรรู้เพียง bytes, protocols, tuning, codec และ transport

## 16. Documentation maintenance rule

เมื่อ architecture, public API, supported platform, release gate หรือ validation status เปลี่ยน ให้ sync อย่างน้อย:

```text
README.md
CODE_WALKTHROUGH.md
docs/PLATFORMS.md
docs/ANDROID_VALIDATION.md   # เมื่อเกี่ยวกับ Android
RELEASE.md                    # เมื่อกระทบ publish/release
```

เป้าหมายคือเอกสารต้องสะท้อน code และ validation evidence ปัจจุบัน ไม่ใช่ roadmap เก่า

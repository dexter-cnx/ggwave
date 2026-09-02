# Code Walkthrough — ggwave

เอกสารนี้อธิบายโครงสร้างและ data flow ของ repository `ggwave` ตั้งแต่ Rust core ไปจนถึง Flutter และ native Android/Kotlin โดยเน้นว่า core ต้องเป็น universal และไม่ผูกกับ application protocol เช่น Bingo, QR, pairing หรือ game state

## 1. ภาพรวมสถาปัตยกรรม

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

หลักการสำคัญคือ `ggwave-core` ไม่รู้จัก Flutter, Android, microphone, speaker หรือ payload semantics ของ application ใด ๆ

## 2. Rust core — `crates/ggwave-core`

ไฟล์หลักคือ:

```text
crates/ggwave-core/src/lib.rs
```

core เป็น source of truth ของ behavior ที่ binding ทุกภาษาใช้ร่วมกัน ได้แก่:

- protocol IDs
- payload size limit
- codec initialization
- sample rate
- ultrasonic start frequency
- ultrasonic pre-emphasis
- encode/decode
- packet deduplication

### 2.1 Stable protocol IDs

Public API กำหนด protocol IDs คงที่:

```rust
pub enum Protocol {
    AudibleFast = 1,
    UltrasonicFast = 5,
}
```

ค่า `5` เป็น app-facing stable ID ของ repository นี้ ส่วนภายใน map ไป upstream:

```rust
ProtocolId::GGWAVE_PROTOCOL_ULTRASOUND_FAST
```

เหตุผลที่มี mapping layer คือ consumer ไม่ควรผูกกับ enum value ภายในของ upstream library โดยตรง

### 2.2 Tuning

ค่าเริ่มต้น:

```text
ultrasonic start frequency = 12,000 Hz
pre-emphasis gain          = 1.8
sample rate                = 48,000 Hz
```

frequency ที่ยอมรับอยู่ในช่วง:

```text
8,000 .. 19,000 Hz
```

`Tuning::apply()` ตั้งทั้ง RX และ TX ultrasonic start frequency ผ่าน ggwave FFI เพื่อให้ encode และ decode ใช้ profile เดียวกัน

### 2.3 Codec ownership

`Codec` ห่อ `ggwave_rs::GgWave`:

```rust
pub struct Codec {
    inner: GgWave,
    tuning: Tuning,
}
```

upstream `GgWave` เป็น `!Send` / `!Sync` ดังนั้นห้ามถือ instance แล้วโยนข้าม thread แบบทั่วไป นี่เป็นข้อจำกัดเชิงสถาปัตยกรรมที่ binding ต้องเคารพ

### 2.4 Encode

flow:

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

ultrasonic pre-emphasis ปัจจุบันใช้ high-frequency emphasis แบบง่าย:

```rust
(input - 0.85 * previous) * gain
```

แล้ว clamp ให้อยู่ในช่วง `[-1.0, 1.0]`

### 2.5 Decode

receive path รับ mono `f32` PCM แล้วแปลงกลับเป็น little-endian bytes ก่อนส่งให้ upstream decoder:

```text
Float32 PCM
   ↓
f32_to_bytes
   ↓
ggwave decode
   ↓
Option<Vec<u8>>
```

`None` หมายถึง chunk นั้นยังไม่มี complete packet

### 2.6 Deduplication

เมื่อเปิด feature `dedup`, `PacketDeduper` จะจำ payload ที่เพิ่งเห็นใน sliding time window ค่า default 800 ms

จุดนี้เป็น transport-level duplicate suppression เท่านั้น ไม่ตีความ sequence number หรือ application semantics

## 3. JNI bridge — `crates/ggwave-jni`

ไฟล์หลัก:

```text
crates/ggwave-jni/src/lib.rs
```

ปัญหาหลักที่ JNI layer ต้องแก้คือ JVM สามารถเรียก native API จากหลาย threads แต่ `GgWave` ไม่ควรถูกย้ายข้าม threads

แนวทางที่ใช้คือ dedicated Rust worker:

```text
Kotlin/Java thread
      ↓
JNI function
      ↓
mpsc Command
      ↓
Dedicated Rust worker thread
      ↓
Codec instance owned here only
```

### 3.1 Command model

worker รับคำสั่งสามชนิด:

```text
SetFrequency
Encode
Decode
```

แต่ละ command มี one-shot reply channel กลับไปยัง JNI caller

ข้อดีคือ:

- codec อยู่ thread เดียวตลอดอายุ process
- Kotlin เรียกจาก main/background threads ได้
- ไม่ต้องเขียน `unsafe impl Send for GgWave`
- ownership boundary อ่านง่ายและ audit ได้

### 3.2 JNI exports

JNI exports ตรงกับ static native methods ใน Kotlin `GgWave`:

```text
nativeSetUltrasonicFrequency
nativeEncode
nativeDecode
```

Rust errors ถูกแปลงเป็น `IllegalStateException` ฝั่ง JVM

## 4. Kotlin facade — `packages/ggwave_kotlin`

package นี้เป็น Android library ที่ไม่ต้องมี Flutter

compatibility ปัจจุบัน:

```text
minSdk       23
compileSdk   36
AGP          9.0.0
Gradle       >= 9.1
Kotlin mode  built-in Kotlin
ABIs         arm64-v8a, armeabi-v7a, x86_64
```

เปิด built-in Kotlin ด้วย:

```properties
android.builtInKotlin=true
```

และไม่ apply legacy `org.jetbrains.kotlin.android` plugin

### 4.1 Low-level API

`GgWave` เป็น codec facade:

```kotlin
GgWave.setUltrasonicFrequency(12_000f)

val waveform: FloatArray = GgWave.encode(
    data = "hello".encodeToByteArray(),
    protocolId = GgWave.PROTOCOL_ULTRASONIC_FAST,
    volume = 85,
)

val payload: ByteArray? = GgWave.decode(samples)
```

เหมาะกับ consumer ที่มี audio engine ของตัวเอง

### 4.2 High-level Android audio API

`GgWaveAudio` เป็น convenience transport สำหรับ Android:

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

receive ใช้ `AudioRecord`, playback ใช้ `AudioTrack`

library ใช้ 48 kHz mono PCM เพื่อให้ตรงกับ codec default

### 4.3 Permission ownership

AAR declare:

```text
android.permission.RECORD_AUDIO
```

แต่ runtime permission request เป็นหน้าที่ host app เพราะ library ไม่ควรบังคับ UI/Activity lifecycle policy ให้ consumer

### 4.4 Callback thread

receive callback ทำงานบน capture thread ของ library ถ้าจะ update Android UI ต้อง dispatch กลับ main thread

`MessageListener` เป็น Kotlin `fun interface` จึงใช้ lambda ใน Kotlin ได้ และยังเป็น Java-friendly SAM interface

## 5. Android hardware validation app

อยู่ที่:

```text
examples/ggwave_kotlin_validation
```

จุดประสงค์ของ app นี้คือทดสอบ public consumer path จริง ไม่เรียก Rust/JNI implementation ภายในโดยตรง

validation controls ครอบคลุม:

- microphone permission
- start listening
- stop listening
- Audible Fast
- Ultrasonic 12 kHz
- Ultrasonic 15 kHz
- Ultrasonic 18 kHz
- sent/received counters
- last received payload
- lifecycle stop on pause

ใช้สองเครื่องเพื่อทดสอบ TX/RX สลับกัน และวัด behavior ตามระยะ/orientation

## 6. Dart package — `packages/ggwave_dart`

`ggwave_dart` เป็น pure Dart layer จึงไม่ depend on Flutter หรือ native FFI

หน้าที่หลัก:

- stable protocol model
- tuning model
- sequence deduper
- transport abstraction

แนวคิดคือ application สามารถเขียน business logic โดย depend กับ interface ระดับ Dart ก่อน แล้วเลือก backend จริงภายหลัง

ตัว package ไม่รู้จัก Bingo packet หรือ QR string format

## 7. Flutter package — `packages/ggwave_flutter`

ชื่อ package ที่ตั้งใจ publish คือ:

```text
ggwave_rs_flutter
```

เนื่องจากชื่อ `ggwave_flutter` มี package อื่นใช้บน pub.dev อยู่แล้ว

baseline:

```text
Flutter >= 3.47.0
Dart    >= 3.12.0
FRB      2.8.0
```

### 7.1 Responsibility

Flutter layer รับผิดชอบ:

- Flutter-facing API
- FRB bridge
- native audio lifecycle
- microphone/speaker integration
- platform packaging

ไม่ควร duplicate codec/tuning logic จาก `ggwave-core`

### 7.2 Native platform targets

Tier 1 target plan:

```text
Android
iOS
macOS
Windows
Linux
```

Web เป็น Tier 2 เพราะ browser audio model ต่างจาก native อย่างมีนัยสำคัญ และควรใช้ Web Audio + AudioWorklet + WASM/JS backend แยก

## 8. End-to-end send flow

### Kotlin / Android

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

### Flutter

conceptually:

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

## 9. End-to-end receive flow

### Kotlin / Android

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

## 10. Build and release gates

### Rust + Flutter/Dart

```bash
./tool/release_check.sh
```

### Kotlin Android

```bash
./tool/release_kotlin_check.sh
```

Kotlin gate ตรวจ:

```text
cargo fmt --check
cargo check -p ggwave-jni
cargo clippy -p ggwave-jni -- -D warnings
      ↓
cargo-ndk 3 ABIs
      ↓
AGP 9 + Gradle 9.1
      ↓
release AAR
      ↓
Maven POM
```

GitHub Actions เพิ่มอีกขั้น:

```text
compile standalone validation app
upload AAR artifact
upload validation APK artifact
```

## 11. Validation evidence

Android/Kotlin build gate ที่ยืนยันแล้ว:

```text
workflow: Kotlin Android #17
run id:   33599743730
status:   success
```

สิ่งที่ผ่านแล้ว:

- Rust formatting/check/clippy
- JNI host compile
- Android JNI cross-compile 3 ABIs
- release AAR
- Maven POM
- standalone consumer validation app compile

คำว่า build validated ไม่เท่ากับ hardware validated

สิ่งที่ยังต้องทำบน physical devices:

- microphone permission behavior
- audible roundtrip
- ultrasonic 12 kHz roundtrip
- ultrasonic 15 kHz roundtrip
- ultrasonic 18 kHz roundtrip
- distance/orientation matrix
- background/resume lifecycle
- repeated start/listen/stop cycles
- release-mode application validation

## 12. Release topology

ลำดับที่ตั้งใจ publish:

```text
1. ggwave-core       -> crates.io
2. ggwave_dart       -> pub.dev
3. ggwave_rs_flutter -> pub.dev
4. ggwave-kotlin     -> Maven repository
```

native support crate เช่น `ggwave-jni` ไม่จำเป็นต้องเป็น public user-facing artifact ถ้า Maven build เป็นเจ้าของการ package native libraries

## 13. สิ่งที่ไม่ควรใส่ใน repository นี้

ตัวอย่างของ application-specific logic ที่ควรอยู่ consumer repo:

```text
BINGO:JOIN:<gid>
BINGO:NUM:<gid>:<num>:<seq>
player ID
game state
QR fallback policy
pairing token format
application retry semantics
```

`ggwave` ควรรู้เพียง bytes, protocols, tuning และ transport

## 14. Maintenance rule

ทุกครั้งที่ architecture, public API, supported platform, release gate หรือ validation status เปลี่ยน ให้ sync อย่างน้อย:

```text
README.md
CODE_WALKTHROUGH.md
docs/PLATFORMS.md
docs/ANDROID_VALIDATION.md   # เมื่อเกี่ยวกับ Android
RELEASE.md                    # เมื่อกระทบ publish/release
```

เป้าหมายคือเอกสารต้องสะท้อน behavior ของ code ปัจจุบัน ไม่ใช่ roadmap เก่า

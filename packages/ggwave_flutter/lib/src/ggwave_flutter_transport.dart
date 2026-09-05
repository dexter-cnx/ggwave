import 'dart:async';
import 'dart:typed_data';

import 'package:ggwave_dart/ggwave_dart.dart';

import 'rust/api.dart' as native;
import 'rust/frb_generated.dart' as frb;

/// Rust-backed Flutter implementation of [GgWaveTransport].
class GgWaveFlutterTransport implements GgWaveTransport {
  GgWaveFlutterTransport({GgWaveTuning tuning = const GgWaveTuning()})
      : _tuning = tuning;

  final GgWaveTuning _tuning;
  final StreamController<Uint8List> _controller =
      StreamController<Uint8List>.broadcast();
  StreamSubscription<List<int>>? _sub;
  bool _initialized = false;

  @override
  Stream<Uint8List> get messages => _controller.stream;

  @override
  Future<void> initialize() async {
    if (_initialized) return;
    _tuning.validate();

    // FRB-generated APIs cannot be called until the generated runtime has
    // loaded the native library and installed its handler. Keep this inside the
    // transport so application consumers do not need to know about RustLib.
    await frb.RustLib.init();

    await native.initRust();
    await native.setUltrasonicFreq(freqStart: _tuning.ultrasonicHz);
    _sub = native
        .createOnMessageStream()
        .listen((e) => _controller.add(Uint8List.fromList(e)));
    _initialized = true;
  }

  @override
  Future<void> setUltrasonicFrequency(double hz) =>
      native.setUltrasonicFreq(freqStart: hz);

  @override
  Future<Float32List> encode(
    Uint8List data, {
    GgWaveProtocol protocol = GgWaveProtocol.audibleFast,
    int volume = 60,
  }) async {
    final effective = protocol.isUltrasonic && volume < 70
        ? _tuning.ultrasonicVolume
        : volume;
    final wave = await native.encode(
      data: data,
      protocolId: protocol.id,
      volume: effective,
    );
    return Float32List.fromList(wave);
  }

  @override
  Future<void> play(Float32List waveform) =>
      native.playWaveform(waveform: waveform);

  @override
  Future<void> startListening({
    GgWaveProtocol protocol = GgWaveProtocol.audibleFast,
  }) =>
      native.startListening(protocolId: protocol.id);

  @override
  Future<void> stopListening() => native.stopListening();

  /// Releases Dart-side subscriptions. Call [stopListening] first if needed.
  Future<void> dispose() async {
    await _sub?.cancel();
    await _controller.close();
  }
}

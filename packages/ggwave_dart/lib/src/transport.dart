import 'dart:typed_data';
import 'protocol.dart';

/// Platform-neutral contract implemented by native/mobile ggwave backends.
abstract interface class GgWaveTransport {
  Stream<Uint8List> get messages;
  Future<void> initialize();
  Future<void> setUltrasonicFrequency(double hz);
  Future<Float32List> encode(
    Uint8List data, {
    GgWaveProtocol protocol = GgWaveProtocol.audibleFast,
    int volume = 60,
  });
  Future<void> play(Float32List waveform);
  Future<void> startListening({
    GgWaveProtocol protocol = GgWaveProtocol.audibleFast,
  });
  Future<void> stopListening();
}

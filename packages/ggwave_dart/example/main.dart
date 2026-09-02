import 'package:ggwave_dart/ggwave_dart.dart';

void main() {
  const tuning = GgWaveTuning();
  tuning.validate();
  print(GgWaveProtocol.ultrasonicFast.id);
}

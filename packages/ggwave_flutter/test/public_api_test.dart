import 'package:flutter_test/flutter_test.dart';
import 'package:ggwave_rs_flutter/ggwave_rs_flutter.dart';

void main() {
  test(
    'ultrasonic protocol is id 5',
    () => expect(GgWaveProtocol.ultrasonicFast.id, 5),
  );
}

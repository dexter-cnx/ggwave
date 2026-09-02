import 'package:ggwave_dart/ggwave_dart.dart';
import 'package:test/test.dart';

void main() {
  test('protocol ids remain wire compatible', () {
    expect(GgWaveProtocol.audibleFast.id, 1);
    expect(GgWaveProtocol.ultrasonicFast.id, 5);
  });
  test('deduper rejects repeated sequence', () {
    final d = SequenceDeduper();
    expect(d.accept(9), isTrue);
    expect(d.accept(9), isFalse);
  });
}

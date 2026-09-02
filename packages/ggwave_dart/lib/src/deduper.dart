import 'dart:typed_data';

/// Sequence-aware deduplicator for hybrid audio/QR delivery.
class SequenceDeduper {
  SequenceDeduper({this.capacity = 256});
  final int capacity;
  final Set<int> _seen = <int>{};
  final List<int> _order = <int>[];

  /// Returns false when a non-0xFF sequence has already been accepted.
  bool accept(int sequence) {
    if (sequence == 0xFF) return true;
    final s = sequence & 0xFF;
    if (_seen.contains(s)) return false;
    _seen.add(s);
    _order.add(s);
    while (_order.length > capacity) {
      _seen.remove(_order.removeAt(0));
    }
    return true;
  }

  void clear() {
    _seen.clear();
    _order.clear();
  }
}

/// Byte-payload equality helper suitable for short packet caches.
bool bytesEqual(Uint8List a, Uint8List b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i] != b[i]) return false;
  }
  return true;
}

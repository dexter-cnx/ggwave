/// Protocol IDs shared by Dart, Flutter and Rust layers.
enum GgWaveProtocol {
  audibleFast(1, false),
  ultrasonicFast(5, true);

  const GgWaveProtocol(this.id, this.isUltrasonic);
  final int id;
  final bool isUltrasonic;
}

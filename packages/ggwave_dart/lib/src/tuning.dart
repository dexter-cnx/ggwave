/// Mobile-oriented ggwave tuning values.
class GgWaveTuning {
  const GgWaveTuning({
    this.ultrasonicHz = 12000,
    this.audibleVolume = 60,
    this.ultrasonicVolume = 85,
    this.dedupWindow = const Duration(milliseconds: 800),
  });
  final double ultrasonicHz;
  final int audibleVolume;
  final int ultrasonicVolume;
  final Duration dedupWindow;

  /// Throws if values are outside supported mobile bounds.
  void validate() {
    if (ultrasonicHz < 8000 || ultrasonicHz > 19000) {
      throw RangeError.range(ultrasonicHz, 8000, 19000, 'ultrasonicHz');
    }
    for (final v in [audibleVolume, ultrasonicVolume]) {
      if (v < 0 || v > 100) throw RangeError.range(v, 0, 100, 'volume');
    }
  }
}

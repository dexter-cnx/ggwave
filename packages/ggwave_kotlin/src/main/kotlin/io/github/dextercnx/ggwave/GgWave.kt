package io.github.dextercnx.ggwave

/**
 * Kotlin facade for the universal Rust ggwave core.
 *
 * The Rust implementation owns a dedicated codec thread, so calls may originate
 * from Android main/background threads without moving the non-Send native codec
 * across threads.
 */
object GgWave {
    const val PROTOCOL_AUDIBLE_FAST: Int = 1
    const val PROTOCOL_ULTRASONIC_FAST: Int = 5

    init {
        System.loadLibrary("ggwave_jni")
    }

    /** Sets the ultrasonic protocol start frequency (8–19 kHz). */
    @JvmStatic
    fun setUltrasonicFrequency(hz: Float) {
        nativeSetUltrasonicFrequency(hz)
    }

    /** Encodes arbitrary application bytes to normalized mono Float samples. */
    @JvmStatic
    fun encode(
        data: ByteArray,
        protocolId: Int = PROTOCOL_AUDIBLE_FAST,
        volume: Int = 60,
    ): FloatArray = nativeEncode(data, protocolId, volume)

    /**
     * Feeds mono Float samples into the streaming decoder.
     * Returns a payload only when a complete packet has been decoded.
     */
    @JvmStatic
    fun decode(samples: FloatArray): ByteArray? = nativeDecode(samples)

    @JvmStatic private external fun nativeSetUltrasonicFrequency(hz: Float): Boolean
    @JvmStatic private external fun nativeEncode(
        data: ByteArray,
        protocolId: Int,
        volume: Int,
    ): FloatArray
    @JvmStatic private external fun nativeDecode(samples: FloatArray): ByteArray?
}

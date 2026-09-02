package io.github.dextercnx.ggwave

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import android.os.SystemClock
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/** Android microphone/speaker convenience layer for [GgWave]. */
object GgWaveAudio {
    const val SAMPLE_RATE_HZ: Int = 48_000

    private val listening = AtomicBoolean(false)
    @Volatile private var captureThread: Thread? = null
    @Volatile private var audioRecord: AudioRecord? = null

    /** Whether microphone capture is currently active. */
    @JvmStatic
    fun isListening(): Boolean = listening.get()

    /**
     * Starts microphone capture and forwards complete decoded packets to [listener].
     *
     * The host application must grant `android.permission.RECORD_AUDIO` before
     * calling this method. The listener executes on the capture thread. Kotlin
     * callers can pass a lambda; Java callers can use [MessageListener].
     */
    @JvmStatic
    fun startListening(listener: MessageListener) {
        if (!listening.compareAndSet(false, true)) return

        val minBufferBytes = AudioRecord.getMinBufferSize(
            SAMPLE_RATE_HZ,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_FLOAT,
        )
        require(minBufferBytes > 0) { "48 kHz PCM_FLOAT recording is not supported" }

        val recorder = AudioRecord.Builder()
            .setAudioSource(MediaRecorder.AudioSource.DEFAULT)
            .setAudioFormat(
                AudioFormat.Builder()
                    .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                    .setSampleRate(SAMPLE_RATE_HZ)
                    .setChannelMask(AudioFormat.CHANNEL_IN_MONO)
                    .build(),
            )
            .setBufferSizeInBytes(maxOf(minBufferBytes, 4096 * Float.SIZE_BYTES))
            .build()

        check(recorder.state == AudioRecord.STATE_INITIALIZED) {
            recorder.release()
            listening.set(false)
            "Unable to initialize AudioRecord"
        }

        audioRecord = recorder
        captureThread = thread(name = "ggwave-audio-capture", isDaemon = true) {
            val buffer = FloatArray(4096)
            try {
                recorder.startRecording()
                while (listening.get()) {
                    val count = recorder.read(buffer, 0, buffer.size, AudioRecord.READ_BLOCKING)
                    if (count <= 0) continue
                    GgWave.decode(buffer.copyOf(count))?.let(listener::onMessage)
                }
            } finally {
                runCatching {
                    if (recorder.recordingState == AudioRecord.RECORDSTATE_RECORDING) {
                        recorder.stop()
                    }
                }
                recorder.release()
                if (audioRecord === recorder) audioRecord = null
                listening.set(false)
            }
        }
    }

    /** Stops a microphone capture started by [startListening]. */
    @JvmStatic
    fun stopListening() {
        if (!listening.getAndSet(false)) return
        runCatching { audioRecord?.stop() }
        captureThread?.interrupt()
        captureThread = null
    }

    /**
     * Plays normalized mono float samples once at 48 kHz.
     *
     * A static AudioTrack is used because a ggwave packet is a finite waveform.
     * The playback thread waits for the playback head to consume the full packet
     * before releasing the track, preventing the waveform tail from being cut off.
     */
    @JvmStatic
    fun play(waveform: FloatArray) {
        require(waveform.isNotEmpty()) { "waveform must not be empty" }

        thread(name = "ggwave-audio-playback", isDaemon = true) {
            val format = AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_FLOAT)
                .setSampleRate(SAMPLE_RATE_HZ)
                .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                .build()
            val attributes = AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                .build()
            val bufferBytes = waveform.size * Float.SIZE_BYTES
            val track = AudioTrack.Builder()
                .setAudioAttributes(attributes)
                .setAudioFormat(format)
                .setTransferMode(AudioTrack.MODE_STATIC)
                .setBufferSizeInBytes(bufferBytes)
                .build()

            try {
                check(track.state == AudioTrack.STATE_INITIALIZED) {
                    "Unable to initialize AudioTrack"
                }

                var offset = 0
                while (offset < waveform.size) {
                    val written = track.write(
                        waveform,
                        offset,
                        waveform.size - offset,
                        AudioTrack.WRITE_BLOCKING,
                    )
                    check(written > 0) { "AudioTrack write failed: $written" }
                    offset += written
                }

                track.play()
                val expectedMs = ((waveform.size.toLong() * 1000L) / SAMPLE_RATE_HZ) + 500L
                val deadline = SystemClock.elapsedRealtime() + expectedMs
                while (
                    track.playState == AudioTrack.PLAYSTATE_PLAYING &&
                    track.playbackHeadPosition.toLong() < waveform.size.toLong() &&
                    SystemClock.elapsedRealtime() < deadline
                ) {
                    SystemClock.sleep(5L)
                }
            } finally {
                runCatching { track.stop() }
                track.release()
            }
        }
    }

    /** Encodes and immediately plays one application payload. */
    @JvmStatic
    fun send(
        data: ByteArray,
        protocolId: Int = GgWave.PROTOCOL_AUDIBLE_FAST,
        volume: Int = 60,
    ) {
        play(GgWave.encode(data, protocolId, volume))
    }
}

package io.github.dextercnx.ggwave

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/** Android microphone/speaker convenience layer for [GgWave]. */
object GgWaveAudio {
    const val SAMPLE_RATE_HZ: Int = 48_000

    private val listening = AtomicBoolean(false)
    @Volatile private var captureThread: Thread? = null
    @Volatile private var audioRecord: AudioRecord? = null

    /**
     * Starts microphone capture and forwards complete decoded packets to [onMessage].
     *
     * The host application must grant `android.permission.RECORD_AUDIO` before
     * calling this method. [onMessage] executes on the capture thread.
     */
    @JvmStatic
    fun startListening(onMessage: (ByteArray) -> Unit) {
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
                    GgWave.decode(buffer.copyOf(count))?.let(onMessage)
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
     * Playback occurs on a dedicated short-lived background thread.
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
            val minBufferBytes = AudioTrack.getMinBufferSize(
                SAMPLE_RATE_HZ,
                AudioFormat.CHANNEL_OUT_MONO,
                AudioFormat.ENCODING_PCM_FLOAT,
            )
            require(minBufferBytes > 0) { "48 kHz PCM_FLOAT playback is not supported" }

            val track = AudioTrack.Builder()
                .setAudioAttributes(attributes)
                .setAudioFormat(format)
                .setTransferMode(AudioTrack.MODE_STREAM)
                .setBufferSizeInBytes(maxOf(minBufferBytes, 4096 * Float.SIZE_BYTES))
                .build()

            try {
                check(track.state == AudioTrack.STATE_INITIALIZED) {
                    "Unable to initialize AudioTrack"
                }
                track.play()
                var offset = 0
                while (offset < waveform.size) {
                    val written = track.write(
                        waveform,
                        offset,
                        waveform.size - offset,
                        AudioTrack.WRITE_BLOCKING,
                    )
                    if (written <= 0) break
                    offset += written
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

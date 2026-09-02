package io.github.dextercnx.ggwave.validation

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.os.Bundle
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import io.github.dextercnx.ggwave.GgWave
import io.github.dextercnx.ggwave.GgWaveAudio
import java.util.concurrent.atomic.AtomicInteger

class MainActivity : Activity() {
    private lateinit var status: TextView
    private val receivedCount = AtomicInteger(0)
    private val sentCount = AtomicInteger(0)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        title = "ggwave validation"
        setContentView(buildUi())
        refreshStatus("Ready")
    }

    override fun onPause() {
        super.onPause()
        GgWaveAudio.stopListening()
        refreshStatus("Listening stopped on pause")
    }

    private fun buildUi(): ScrollView {
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(32, 32, 32, 32)
        }

        status = TextView(this).apply {
            textSize = 16f
            setPadding(0, 0, 0, 24)
        }
        root.addView(status)

        root.addButton("Grant microphone permission") { requestMicPermission() }
        root.addButton("Start listening") { startListening() }
        root.addButton("Stop listening") {
            GgWaveAudio.stopListening()
            refreshStatus("Listening stopped")
        }
        root.addButton("Send audible") {
            send(GgWave.PROTOCOL_AUDIBLE_FAST, null)
        }
        root.addButton("Send ultrasonic 12 kHz") {
            send(GgWave.PROTOCOL_ULTRASONIC_FAST, 12_000f)
        }
        root.addButton("Send ultrasonic 15 kHz") {
            send(GgWave.PROTOCOL_ULTRASONIC_FAST, 15_000f)
        }
        root.addButton("Send ultrasonic 18 kHz") {
            send(GgWave.PROTOCOL_ULTRASONIC_FAST, 18_000f)
        }

        return ScrollView(this).apply { addView(root) }
    }

    private fun LinearLayout.addButton(label: String, action: () -> Unit) {
        addView(Button(this@MainActivity).apply {
            text = label
            setOnClickListener { action() }
        })
    }

    private fun requestMicPermission() {
        if (checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED) {
            refreshStatus("Microphone permission already granted")
            return
        }
        requestPermissions(arrayOf(Manifest.permission.RECORD_AUDIO), REQUEST_MIC)
    }

    private fun startListening() {
        if (checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            requestMicPermission()
            return
        }

        GgWaveAudio.startListening { payload ->
            val count = receivedCount.incrementAndGet()
            val printable = payload.decodeToString().replace("\n", "\\n")
            runOnUiThread {
                refreshStatus("Received #$count: $printable")
            }
        }
        refreshStatus("Listening")
    }

    private fun send(protocolId: Int, ultrasonicHz: Float?) {
        ultrasonicHz?.let(GgWave::setUltrasonicFrequency)
        val sequence = sentCount.incrementAndGet()
        val label = ultrasonicHz?.toInt()?.let { "${it}Hz" } ?: "audible"
        val payload = "GGWAVE_VALIDATE:$label:$sequence".encodeToByteArray()
        GgWaveAudio.send(
            data = payload,
            protocolId = protocolId,
            volume = if (ultrasonicHz == null) 60 else 85,
        )
        refreshStatus("Sent #$sequence: ${payload.decodeToString()}")
    }

    private fun refreshStatus(message: String) {
        status.text = buildString {
            appendLine(message)
            appendLine("permission=${checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED}")
            appendLine("listening=${GgWaveAudio.isListening()}")
            appendLine("sent=${sentCount.get()} received=${receivedCount.get()}")
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == REQUEST_MIC) {
            val granted = grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED
            refreshStatus(if (granted) "Microphone permission granted" else "Microphone permission denied")
        }
    }

    companion object {
        private const val REQUEST_MIC = 1001
    }
}

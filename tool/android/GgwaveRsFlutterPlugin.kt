package com.dextercnx.ggwave

import android.content.Context
import io.flutter.embedding.engine.plugins.FlutterPlugin

/**
 * Android-only bootstrap for native audio dependencies that require a valid
 * application Context before CPAL/Oboe is touched from Rust.
 */
class GgwaveRsFlutterPlugin : FlutterPlugin {
    external fun nativeInitializeAndroidContext(context: Context)

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        System.loadLibrary("ggwave_flutter_native")
        nativeInitializeAndroidContext(binding.applicationContext)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) = Unit
}

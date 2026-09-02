package io.github.dextercnx.ggwave

/** Java-friendly callback for decoded ggwave payloads. */
fun interface MessageListener {
    fun onMessage(payload: ByteArray)
}

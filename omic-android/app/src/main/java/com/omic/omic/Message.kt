package com.omic.omic

// Must match Rust one
enum class ClientRequests(val byteValue: Byte) {
    DISCONNECT(0),
    CONNECT(1),
    HELLO(2),
}

enum class MicrophoneServiceAction(val value: Int) {
    CREATED_SERVICE(0),
    START_SERVICE(1),
    STOP_SERVICE(2),
}

const val NOTIFICATION_ID = 7312
const val AUDIO_BUFFER_SIZE = 608
const val SERVER_PORT = 8888

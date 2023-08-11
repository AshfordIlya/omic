package com.omic.omic

// Must match Rust one
enum class UdpSocketMessage(val byteValue: Byte) {
    CONNECT(1),
    DISCONNECT(0)
}

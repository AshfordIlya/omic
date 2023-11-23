package com.omic.omic

interface ServiceCallbacks {
    fun onConnect(info: ConnectionInfo): Unit
    fun onDisconnect(): Unit
}

package com.omic.omic

import android.annotation.SuppressLint
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Binder
import android.os.IBinder
import android.util.Log
import androidx.core.content.getSystemService
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.BoundDatagramSocket
import io.ktor.network.sockets.Datagram
import io.ktor.network.sockets.InetSocketAddress
import io.ktor.network.sockets.ServerSocket
import io.ktor.network.sockets.Socket
import io.ktor.network.sockets.aSocket
import io.ktor.network.sockets.isClosed
import io.ktor.network.sockets.openReadChannel
import io.ktor.network.sockets.openWriteChannel
import io.ktor.network.sockets.toJavaAddress
import io.ktor.util.network.hostname
import io.ktor.utils.io.ByteReadChannel
import io.ktor.utils.io.ByteWriteChannel
import io.ktor.utils.io.cancel
import io.ktor.utils.io.close
import io.ktor.utils.io.core.BytePacketBuilder
import io.ktor.utils.io.core.writeFully
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelChildren
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.launch
import kotlinx.coroutines.newSingleThreadContext
import java.util.concurrent.atomic.AtomicBoolean

class MicrophoneService : Service() {
    private lateinit var udpSocket: BoundDatagramSocket
    private lateinit var tcpSocket: ServerSocket
    private var serverScope = ServerScope()
    private lateinit var serverConnection: ServerConnection
    private lateinit var connectionCallback: ServiceCallbacks
    private lateinit var notification: Notification
    private var serviceStarted = false
    var micMuted = AtomicBoolean(false)

    @SuppressLint("MissingPermission")
    private val audioRecord = AudioRecord(
        MediaRecorder.AudioSource.VOICE_PERFORMANCE,
        48000,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT,
        AudioRecord.getMinBufferSize(48000,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT)
    )

    private fun startServer() {
        serverScope.launch {
            serverConnection.readSocket()
        }
    }

    private fun buildNotification() {
        val notificationId = "omic"
        val channel1 = NotificationChannel(
            notificationId,
            "omic microphone",
            NotificationManager.IMPORTANCE_HIGH
        )
        val manager = getSystemService<NotificationManager>()
        manager?.createNotificationChannel(channel1)
        notification = Notification.Builder(this, notificationId)
            .setContentTitle("omic")
            .build()
    }

    private fun bindNetworkSockets() {
        val selectorManager = SelectorManager(Dispatchers.IO)
        udpSocket = aSocket(selectorManager).udp().bind(InetSocketAddress("0.0.0.0", 0))
        tcpSocket = aSocket(selectorManager).tcp().bind(InetSocketAddress("0.0.0.0", SERVER_PORT))
    }


    override fun onCreate() {
        super.onCreate()
        Log.i("omic", "Created Microphone Service")
        buildNotification()
        bindNetworkSockets()
        serverConnection = ServerConnection()

    }

    override fun onStartCommand(intent: Intent, flags: Int, startId: Int): Int {
        Log.i("omic", "Started Microphone Service")
        if (!serviceStarted) {
            serviceStarted = true
            startForeground(
                NOTIFICATION_ID,
                notification,
                FOREGROUND_SERVICE_TYPE_MICROPHONE
            )
            startServer()
        }
        return START_STICKY
    }

    fun setConnectCallback(callback: ServiceCallbacks) {
        connectionCallback = callback
    }


    inner class MicrophoneBinder : Binder() {
        fun getService(): MicrophoneService = this@MicrophoneService
        fun disconnectServer() {
            serverConnection.disconnect()
        }
    }

    override fun onBind(intent: Intent): IBinder {
        return MicrophoneBinder()
    }

    override fun onDestroy() {
        serverConnection.disconnect()
        serverScope.coroutineContext.cancelChildren()
        serverScope.cancel()
        tcpSocket.close()
        udpSocket.close()
        stopForeground(STOP_FOREGROUND_REMOVE)
        super.onDestroy()
    }

    private inner class ServerConnection {
        private var isConnected = false
        private lateinit var socket: Socket
        private lateinit var readChannel: ByteReadChannel
        private lateinit var writeChannel: ByteWriteChannel
        private lateinit var udpAddress: InetSocketAddress
        val buffer = ByteArray(AudioRecord.getMinBufferSize(48000,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT))

        @OptIn(ExperimentalCoroutinesApi::class, DelicateCoroutinesApi::class)
        suspend fun readSocket() {
            while (!tcpSocket.isClosed) {
                socket = tcpSocket.accept()
                isConnected = true
                readChannel = socket.openReadChannel()
                writeChannel = socket.openWriteChannel()
                while (!readChannel.isClosedForRead) {
                    var byteRead: Byte? = null
                    try {
                        byteRead = readChannel.readByte()
                    } catch (e: ClosedReceiveChannelException) {
                        onDisconnect()
                    }
                    if (byteRead != null)
                        when (byteRead) {
                            ClientRequests.CONNECT.byteValue -> {
                                udpAddress = InetSocketAddress(
                                    socket.remoteAddress.toJavaAddress().hostname,
                                    readChannel.readShort().toUShort().toInt()
                                )
                                serverScope.launch(newSingleThreadContext("audio thread")) { sendAudio() }
                                onConnect()
                            }

                            ClientRequests.DISCONNECT.byteValue -> {
                                isConnected = false
                                readChannel.cancel()
                                writeChannel.close()
                                socket.close()
                                onDisconnect()
                            }

                            ClientRequests.HELLO.byteValue -> {
                                writeChannel.writeByte(ClientRequests.HELLO.byteValue)
                                writeChannel.flush()
                            }
                        }
                }
            }
        }

        suspend fun sendAudio() {
            audioRecord.startRecording()
             audioRecord.read(
                    buffer,
                    0,
                    buffer.size,
                    AudioRecord.READ_NON_BLOCKING
                )
            var read = 0
            while (isConnected) {
                read = audioRecord.read(
                    buffer,
                    0,
                    buffer.size,
                    AudioRecord.READ_NON_BLOCKING
                )

                if (read > 0) {
                    Log.i("omic", "bytes sent -> $read")
                    val builder = BytePacketBuilder()
                    builder.writeFully(buffer, 0, read)
                    udpSocket.send(
                        Datagram(
                            builder.build(),
                            udpAddress
                        )
                    )
                }
            }
            audioRecord.stop()
        }

        fun disconnect() {
            if (isConnected) {
                readChannel.cancel(ClosedReceiveChannelException(null))
            }
        }

        fun onDisconnect() {
            isConnected = false
            writeChannel.close()
            socket.close()
            connectionCallback.onDisconnect()
        }

        fun onConnect() {
            connectionCallback.onConnect(ConnectionInfo(udpAddress.hostname + ":" + udpAddress.port))
        }
    }


    private inner class ServerScope : CoroutineScope {
        override val coroutineContext = Dispatchers.IO + SupervisorJob()
        fun cancel() {
            coroutineContext.cancel()
        }
    }
}

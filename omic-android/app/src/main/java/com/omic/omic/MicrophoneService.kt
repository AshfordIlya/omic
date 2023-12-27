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
import io.ktor.util.network.hostname
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress
import java.net.ServerSocket
import java.net.Socket
import java.util.concurrent.atomic.AtomicBoolean

class MicrophoneService : Service() {
    private lateinit var udpSocket: DatagramSocket
    private lateinit var tcpSocket: ServerSocket
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
        AudioRecord.getMinBufferSize(
            48000,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_16BIT
        )
    )


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
        udpSocket = DatagramSocket(0)
        tcpSocket = ServerSocket(8888)
    }

    private fun startServer() {
        serverConnection.readSocket()
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
            Log.i("omic", "began thread")
            Thread (
                 Runnable {
                    startServer()
                 }
            ).start()
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
        tcpSocket.close()
        udpSocket.close()
        stopForeground(STOP_FOREGROUND_REMOVE)
        super.onDestroy()
    }

    private inner class ServerConnection: Runnable {
        private var isConnected = false
        private lateinit var socket: Socket
        private lateinit var readChannel: InputStream
        private lateinit var writeChannel: OutputStream
        private lateinit var udpAddress: InetSocketAddress
        val buffer = ByteArray(
            AudioRecord.getMinBufferSize(
                48000,
                AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT
            )
        )

        override fun run() {
            Log.i("omic", "run")
            readSocket()
        }



        fun readSocket() {
            while (!tcpSocket.isClosed) {
                 Log.i("omic", "test")
                socket = tcpSocket.accept()
                Log.i("omic", "Found connection")
                isConnected = true
                readChannel = socket.getInputStream()
                writeChannel = socket.getOutputStream()
                while (!socket.isClosed) {
                    var byteRead = -1
                    try {
                        byteRead = readChannel.read()
                    } catch (e: IOException) {
                        Log.i("omic", "disconnected?")
                        onDisconnect()
                    }
                    if (byteRead != -1)
                        when (byteRead) {
                            ClientRequests.CONNECT.byteValue -> {
                                val msb = readChannel.read().toUInt() shl 8
                                val lsb = readChannel.read().toUInt() + msb
                                val port = lsb.toInt()
                                Log.i("omic", "$port UDP")
                                //val port = .toUShort().toUShort().toInt()
                                udpAddress = InetSocketAddress(
                                    socket.inetAddress.hostAddress,
                                    port
                                )
                                Log.i("omic", "$port")
                                Thread (
                                    Runnable {
                                        sendAudio()
                                    }
                                ).start()
                                Log.i("omic", "connect byte")
                                onConnect()
                            }

                            ClientRequests.DISCONNECT.byteValue -> {
                                onDisconnect()
                            }

                            ClientRequests.HELLO.byteValue -> {
                                writeChannel.write(ClientRequests.HELLO.byteValue)
                                writeChannel.flush()
                            }
                        }
                }
                Log.i("omic", "Finished Loop")
            }
        }

         fun sendAudio() {
            audioRecord.startRecording()
            audioRecord.read(
                buffer,
                0,
                buffer.size,
                AudioRecord.READ_NON_BLOCKING
            )
            var read = 0
            while (isConnected) {
//                Log.i("omic", "testing")
                read = audioRecord.read(
                    buffer,
                    0,
                    buffer.size,
                    AudioRecord.READ_NON_BLOCKING
                )
               if (read > 0) {
                    val packet = DatagramPacket(buffer, read, udpAddress)
                   udpSocket.send(packet)

               }
            }
            audioRecord.stop()
        }

        fun disconnect() {
            if (isConnected) {
                readChannel.close()
            }
        }

        fun onDisconnect() {
            isConnected = false
            socket.close()
            connectionCallback.onDisconnect()
        }

        fun onConnect() {
            connectionCallback.onConnect(ConnectionInfo(udpAddress.hostname + ":" + udpAddress.port))
        }
    }
}

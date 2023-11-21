package com.omic.omic

import android.annotation.SuppressLint
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Binder
import android.os.Handler
import android.os.HandlerThread
import android.os.IBinder
import android.os.Looper
import android.os.Message
import android.os.Process
import android.util.Log
import android.widget.Toast
import androidx.core.content.getSystemService
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.Datagram
import io.ktor.network.sockets.InetSocketAddress
import io.ktor.network.sockets.aSocket
import io.ktor.utils.io.core.BytePacketBuilder
import io.ktor.utils.io.core.writeFully
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicBoolean


class MicrophoneService : Service() {
    val port = 8888
    val micMuted = AtomicBoolean(false)

    private var serviceLooper: Looper? = null
    private var serviceHandler: ServiceHandler? = null
    private val notificationChannelId = "omic"
    private val notificationId = 123
    private val binder = MicrophoneBinder()
    private val selectorManager = SelectorManager(Dispatchers.IO)
    private val serverSocket = aSocket(selectorManager)
        .udp()
        .bind(InetSocketAddress("0.0.0.0", port))
    private val isConnected = AtomicBoolean(false)
    private val sampleRate = 48000
    private val bufferSize = 768
    private val job = SupervisorJob()
    private val scope = CoroutineScope(Dispatchers.IO + job)

    @SuppressLint("MissingPermission")
    private val audioRecord = AudioRecord(
        MediaRecorder.AudioSource.MIC,
        sampleRate,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT,
        bufferSize
    )

    private inner class ServiceHandler(looper: Looper) : Handler(looper) {
        override fun handleMessage(msg: Message) {
            val buffer = ByteArray(bufferSize)

            scope.launch {
                while (true) {
                    val datagram = serverSocket.receive()
                    val byte = datagram.packet.readByte()
                    if (byte == UdpSocketMessage.CONNECT.byteValue) {
                        audioRecord.startRecording()
                        isConnected.set(true)

                        scope.launch {
                            while (true) {
                                val incomingDatagram = serverSocket.receive()
                                val incomingByte = incomingDatagram.packet.readByte()
                                if (incomingByte == UdpSocketMessage.DISCONNECT.byteValue) {
                                    isConnected.set(false)
                                    audioRecord.stop()
                                    break
                                }
                            }
                        }
                    }

                    while (isConnected.get()) {
                        audioRecord.read(
                            buffer,
                            0,
                            buffer.size,
                            AudioRecord.READ_BLOCKING
                        )

                        if (!micMuted.get()) {
                            val builder = BytePacketBuilder()
                            builder.writeFully(buffer)
                            serverSocket.send(
                                Datagram(
                                    builder.build(),
                                    datagram.address
                                )
                            )
                        }
                    }
                }
            }
        }
    }

    override fun onCreate() {
        HandlerThread("omic microphone", Process.THREAD_PRIORITY_URGENT_AUDIO).apply {
            start()
            serviceLooper = looper
            serviceHandler = ServiceHandler(looper)
        }
    }

    override fun onStartCommand(intent: Intent, flags: Int, startId: Int): Int {
        Toast.makeText(this, "service starting", Toast.LENGTH_SHORT).show()
        val channel1 = NotificationChannel(
            notificationChannelId,
            "omic microphone",
            NotificationManager.IMPORTANCE_HIGH
        )
        val manager = getSystemService<NotificationManager>()
        manager?.createNotificationChannel(channel1)
        val notification = Notification.Builder(this, notificationChannelId)
            .setContentTitle("omic")
            .build()

        startForeground(notificationId, notification)
        serviceHandler?.obtainMessage()?.also { msg ->
            msg.arg1 = startId
            serviceHandler?.sendMessage(msg)
        }

        return START_STICKY
    }

    inner class MicrophoneBinder : Binder() {
        fun getService(): MicrophoneService = this@MicrophoneService
    }

    override fun onBind(intent: Intent): IBinder {
        return binder
    }

    override fun onDestroy() {
        isConnected.set(false)
        audioRecord.stop()
        serverSocket.close()
    }
}

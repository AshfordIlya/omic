package com.omic.omic

import android.annotation.SuppressLint
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Bundle
import android.os.PowerManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.getSystemService
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.lifecycleScope
import io.ktor.network.selector.SelectorManager
import io.ktor.network.sockets.BoundDatagramSocket
import io.ktor.network.sockets.Datagram
import io.ktor.network.sockets.InetSocketAddress
import io.ktor.network.sockets.aSocket
import io.ktor.utils.io.core.BytePacketBuilder
import io.ktor.utils.io.core.writeFully
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.concurrent.atomic.AtomicBoolean

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val requestPermissionLauncher =
            registerForActivityResult(
                ActivityResultContracts.RequestPermission()
            ) { isGranted: Boolean ->
                if (isGranted) {
                    onPermissionGranted()
                } else {
                    setContent {
                        PermissionRequiredDialog()
                    }
                }
            }

        requestPermissionLauncher.launch(android.Manifest.permission.RECORD_AUDIO)
    }

    private fun setupWifiCallbacks(ipAddressState: MutableLiveData<String?>) {
        val cm = this.getSystemService<ConnectivityManager>()
        val networkRequestBuilder = NetworkRequest.Builder()
        networkRequestBuilder.addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
        val networkRequest = networkRequestBuilder.build()

        cm?.registerNetworkCallback(networkRequest, object :
            ConnectivityManager.NetworkCallback() {
            override fun onAvailable(network: Network) {
                ipAddressState.postValue(cm.getIpv4Address(network))
            }

            override fun onLost(network: Network) {
                ipAddressState.postValue(null)
            }
        })
    }

    private fun getInitialIpAddress(): String? {
        val cm = this.getSystemService<ConnectivityManager>()
        val isWifi =
            cm?.getNetworkCapabilities(cm.activeNetwork)?.capabilities?.contains(
                NetworkCapabilities.TRANSPORT_WIFI
            ) == true

        return if (isWifi) cm.getIpv4Address(cm?.activeNetwork) else null
    }

    private suspend fun handleDisconnectThread(
        serverSocket: BoundDatagramSocket,
        audioRecord: AudioRecord,
        isConnected: AtomicBoolean
    ) {
        serverSocket.let {
            while (true) {
                val incomingDatagram = serverSocket.receive()
                incomingDatagram.let {
                    val incomingByte = it.packet.readByte()
                    if (incomingByte == UdpSocketMessage.DISCONNECT.byteValue) {
                        isConnected.set(false)
                        audioRecord.stop()
                    }
                }
            }
        }
    }

    private suspend fun runRecordingThread(
        serverSocket: BoundDatagramSocket,
        audioRecord: AudioRecord,
        buffer: ByteArray,
        micMuted: AtomicBoolean,
        isConnected: AtomicBoolean
    ) {
        serverSocket.use {
            while (true) {
                val datagram = serverSocket.receive()
                val byte = datagram.packet.readByte()
                if (byte == UdpSocketMessage.CONNECT.byteValue) {
                    audioRecord.startRecording()
                    isConnected.set(true)
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

    private fun onPermissionGranted() {
        val ipAddress = getInitialIpAddress()
        val ipAddressState = MutableLiveData(ipAddress)
        setupWifiCallbacks(ipAddressState)

        if (this.getSystemService<PowerManager>()?.isSustainedPerformanceModeSupported == true) {
            this.window.setSustainedPerformanceMode(true)
        }

        val micMuted = AtomicBoolean(false)

        val selectorManager = SelectorManager(Dispatchers.IO)
        val serverSocket = aSocket(selectorManager)
            .udp()
            .bind(InetSocketAddress("0.0.0.0", 8888))
        val isConnected = AtomicBoolean(false)
        val sampleRate = 48000
        val bufferSize = 768

        @SuppressLint("MissingPermission")
        val audioRecord = AudioRecord(
            MediaRecorder.AudioSource.MIC,
            sampleRate,
            AudioFormat.CHANNEL_IN_MONO,
            AudioFormat.ENCODING_PCM_16BIT,
            bufferSize
        )

        val buffer = ByteArray(bufferSize)

        lifecycleScope.launch {
            withContext(Dispatchers.IO) {
                runRecordingThread(serverSocket, audioRecord, buffer, micMuted, isConnected)
            }
        }

        lifecycleScope.launch {
            withContext(Dispatchers.IO) {
                handleDisconnectThread(serverSocket, audioRecord, isConnected)
            }
        }

        setContent {
            MainUI(
                onMicrophoneChange = { isMuted ->
                    micMuted.set(isMuted)
                },
                ipAddress = ipAddressState
            )
        }
    }
}

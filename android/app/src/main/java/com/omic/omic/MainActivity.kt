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
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.MicOff
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.livedata.observeAsState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.core.content.getSystemService
import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.lifecycleScope
import io.ktor.network.selector.SelectorManager
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
                    // TODO: refactor
                    val cm = this.getSystemService<ConnectivityManager>()
                    val isWifi =
                        cm?.getNetworkCapabilities(cm.activeNetwork)?.capabilities?.contains(
                            NetworkCapabilities.TRANSPORT_WIFI
                        ) == true
                    val ipAddress = if (isWifi) cm.getIpv4Address(cm?.activeNetwork) else null

                    val ipAddressState = MutableLiveData(ipAddress)
                    val networkRequestBuilder = NetworkRequest.Builder()
                    networkRequestBuilder.addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
                    val networkRequest = networkRequestBuilder.build()

                    cm?.registerNetworkCallback(networkRequest, object :
                        ConnectivityManager.NetworkCallback() {
                        override fun onAvailable(network: Network) {
                            Log.i("WIFI", "Connected to $network")
                            ipAddressState.postValue(cm.getIpv4Address(network))
                        }

                        override fun onLost(network: Network) {
                            Log.i("WIFI", "Disconnected")
                            ipAddressState.postValue(null)
                        }
                    })

                    if ((this.getSystemService(POWER_SERVICE) as PowerManager).isSustainedPerformanceModeSupported) {
                        this.window.setSustainedPerformanceMode(true)
                    }
                    val micMuted = AtomicBoolean(false)
                    val sampleRate = 48000
                    val bufferSize = 768

                    Log.i("epic", "$sampleRate $bufferSize")

                    lifecycleScope.launch {
                        withContext(Dispatchers.IO) {
                            @SuppressLint("MissingPermission")
                            val audioRecord = AudioRecord(
                                MediaRecorder.AudioSource.MIC,
                                sampleRate,
                                AudioFormat.CHANNEL_IN_MONO,
                                AudioFormat.ENCODING_PCM_16BIT,
                                bufferSize
                            )

                            val buffer = ByteArray(bufferSize)
                            val selectorManager = SelectorManager(Dispatchers.IO)
                            val serverSocket = aSocket(selectorManager).udp()
                                .bind(InetSocketAddress("0.0.0.0", 8888))

                            val isConnected = AtomicBoolean(false)
                            serverSocket.use {
                                while (true) {
                                    Log.i("incoming", "waiting for message")
                                    val datagram = serverSocket.receive()
                                    val byte = datagram.packet.readByte()
                                    Log.i("incoming", "$byte")
                                    if (byte == UdpSocketMessage.CONNECT.byteValue) {
                                        audioRecord.startRecording()
                                        isConnected.set(true)
                                    }

                                    while (isConnected.get()) {
                                        // TODO: do on different thread to save this one for sending
                                        val incomingDatagram = serverSocket.incoming.tryReceive()
                                        incomingDatagram.let {
                                            val incomingByte = it.getOrNull()?.packet?.readByte()
                                            if (incomingByte == UdpSocketMessage.DISCONNECT.byteValue) {
                                                isConnected.set(false)
                                                audioRecord.stop()
                                            }
                                        }

                                        if (isConnected.get()) {
                                            audioRecord.read(
                                                buffer,
                                                0,
                                                bufferSize,
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
                    }

                    setContent {
                        MainUI(
                            onMicrophoneChange = { isMuted ->
                                micMuted.set(isMuted)
                            },
                            ipAddress = ipAddressState
                        )
                    }
                } else {
                    setContent {
                        PermissionRequiredDialog()
                    }
                }
            }

        requestPermissionLauncher.launch(android.Manifest.permission.RECORD_AUDIO)
    }
}

@Composable
fun MainUI(onMicrophoneChange: (isMuted: Boolean) -> Unit, ipAddress: LiveData<String?>) {
    Scaffold(
        topBar = { TopBar() }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterVertically),
        ) {
            val micMuted = remember { mutableStateOf(false) }
            val ipAddressState: String? by ipAddress.observeAsState()
            val text = ipAddressState.let {
                if (it.isNullOrEmpty()) {
                    return@let "Not Connected to WIFI"
                } else {
                    return@let it
                }
            }
            Text(text = text, modifier = Modifier)
            IconButton(onClick = {
                micMuted.value = !micMuted.value
                onMicrophoneChange(micMuted.value)
            }) {
                Icon(
                    modifier = Modifier.size(48.dp),
                    imageVector = if (micMuted.value) Icons.Filled.MicOff else Icons.Filled.Mic,
                    contentDescription = "Mic on",
                )
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun PermissionRequiredDialog() {
    val openDialog = remember { mutableStateOf(true) }
    if (openDialog.value) {

        AlertDialog(
            onDismissRequest = {
                // Dismiss the dialog when the user clicks outside the dialog or on the back
                // button. If you want to disable that functionality, simply use an empty
                // onDismissRequest.
                openDialog.value = false
            },
            title = {
                Text(text = "Microphone permission is required")
            },
            text = {},
            confirmButton = {},
            dismissButton = {
                TextButton(
                    onClick = {
                        openDialog.value = false
                    }
                ) {
                    Text("Dismiss")
                }
            }
        )
    }
}

@SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
fun TopBar() {
    TopAppBar(
        title = {
            Text(
                "omic",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        },
        navigationIcon = {},
        actions = {}
    )
}

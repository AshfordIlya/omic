package com.omic.omic

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Bundle
import android.os.IBinder
import android.os.PowerManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.core.content.getSystemService
import androidx.lifecycle.MutableLiveData

class MainActivity : ComponentActivity() {
    private lateinit var microphoneService: MicrophoneService
    private var serviceBound: MutableState<Boolean> = mutableStateOf(false)

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, service: IBinder) {
            val binder = service as MicrophoneService.MicrophoneBinder
            microphoneService = binder.getService()
            serviceBound.value = true
        }

        override fun onServiceDisconnected(arg0: ComponentName) {
            serviceBound.value = false
        }
    }

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

    private fun onPermissionGranted() {
        val intent = Intent(this, MicrophoneService::class.java).also {
            bindService(
                it,
                connection,
                Context.BIND_AUTO_CREATE
            )
        }

        val ipAddress = getInitialIpAddress()
        val ipAddressState = MutableLiveData(ipAddress)
        setupWifiCallbacks(ipAddressState)

        if (this.getSystemService<PowerManager>()?.isSustainedPerformanceModeSupported == true) {
            this.window.setSustainedPerformanceMode(true)
        }

        startForegroundService(intent)

        setContent {
            if (serviceBound.value) {
                MainUI(
                    onMicrophoneChange = { isMuted ->
                        microphoneService.micMuted.set(isMuted)
                    },
                    ipAddress = ipAddressState,
                    port = microphoneService.port.toString()
                )
            }
        }
    }
}

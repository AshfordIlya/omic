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
import androidx.compose.runtime.livedata.observeAsState
import androidx.core.content.getSystemService
import androidx.lifecycle.MutableLiveData

class MainActivity : ComponentActivity(), ServiceCallbacks {
    private var isConnected: MutableLiveData<Boolean> = MutableLiveData(false)
    private var connectionInfo: MutableLiveData<ConnectionInfo?> = MutableLiveData(null)
    private val connection = object : ServiceConnection {
        override fun onServiceConnected(className: ComponentName, service: IBinder) {
            val binder = service as MicrophoneService.MicrophoneBinder
            val microphoneService = binder.getService()
            microphoneService.setConnectCallback(this@MainActivity)
            val ipAddress = getInitialIpAddress()
            val ipAddressState = MutableLiveData(ipAddress)
            setupWifiCallbacks(ipAddressState)
                
            setContent {
                MainUI(
                    onMicrophoneChange = { isMuted ->
                        microphoneService.micMuted.set(isMuted)
                    },
                    ipAddress = ipAddressState.observeAsState(),
                    isConnected = isConnected.observeAsState(false),
                    connectionInfo = connectionInfo.observeAsState(),
                    onDisconnect = { binder.disconnectServer() }
                )
            }
        }


        override fun onServiceDisconnected(arg0: ComponentName) {
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

    override fun onDestroy() {
        super.onDestroy()
        unbindService(connection)

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
        startForegroundService(intent)
        if (this.getSystemService<PowerManager>()?.isSustainedPerformanceModeSupported == true) {
            this.window.setSustainedPerformanceMode(true)
        }

    }

    override fun onConnect(info: ConnectionInfo) {
        isConnected.postValue(true)
        connectionInfo.postValue(info)
    }

    override fun onDisconnect() {
        isConnected.postValue(false)
    }

}


package com.omic.omic

import android.net.ConnectivityManager
import android.net.Network

fun ConnectivityManager?.getIpv4Address(activeNetwork: Network?): String? {
    return this?.getLinkProperties(activeNetwork)
        ?.linkAddresses
        ?.firstOrNull { linkAddress ->
            linkAddress
                ?.address
                ?.hostAddress
                ?.contains('.') == true
        }
        ?.address
        ?.hostAddress
}

package com.omic.omic

import android.annotation.SuppressLint
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
import androidx.lifecycle.LiveData

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

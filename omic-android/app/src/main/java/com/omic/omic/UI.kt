package com.omic.omic

import android.annotation.SuppressLint
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.sharp.ArrowBack
import androidx.compose.material.icons.sharp.ArrowForward
import androidx.compose.material.icons.sharp.Mic
import androidx.compose.material.icons.sharp.MicOff
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
@Composable
fun MainUI(
    onMicrophoneChange: (isMuted: Boolean) -> Unit, ipAddress: State<String?>, port: String, isConnected: State<Boolean>, connectionInfo: State<ConnectionInfo?>,
    onDisconnect: () -> Unit
) {
    val micMuted = remember { mutableStateOf(false) }
    val buttonColours = ButtonDefaults.buttonColors(
        containerColor = if (micMuted.value) MaterialTheme.colorScheme.errorContainer else Color(
            201,
            238,
            158
        ),
        contentColor = if (micMuted.value) MaterialTheme.colorScheme.onErrorContainer else Color(
            73,
            103,
            39
        )
    )

    Scaffold(topBar = { TopBar() }) {
        Column(modifier = Modifier.fillMaxHeight()) {
            ElevatedCard(
                colors = CardDefaults.cardColors(
                    containerColor = Color.White,
                ),
                modifier = Modifier
                    .padding(24.dp)
                    .offset(y = 48.dp)
                    .fillMaxWidth(),
                elevation = CardDefaults.cardElevation(
                    defaultElevation = 2.dp
                ),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Icon(
                            Icons.Sharp.ArrowBack,
                            contentDescription = null,
                            modifier = Modifier.padding(top = 16.dp, start = 16.dp),
                            tint = Color.Gray
                        )
                        Text(
                            text = "Host",
                            color = Color.Gray,
                            modifier = Modifier.padding(start = 16.dp)
                        )
                        Text(
                            text = ipAddress.value ?: "Not connected",
                            fontSize = 16.sp,
                            fontWeight = FontWeight.SemiBold,
                            modifier = Modifier.padding(start = 16.dp, bottom = 16.dp)
                        )
                    }

                    Column {
                        ElevatedButton(
                            onClick = {
                                onMicrophoneChange(!micMuted.value)
                                micMuted.value = !micMuted.value
                            },
                            colors = buttonColours,
                            modifier = Modifier.padding(end = 16.dp),
                        ) {
                            Icon(
                                if (micMuted.value) Icons.Sharp.MicOff else Icons.Sharp.Mic,
                                contentDescription = null,
                            )
                        }
                    }
                }

                AnimatedVisibility(isConnected.value) {
                    Divider(
                        thickness = Dp.Hairline,
                        modifier = Modifier.padding(start = 16.dp, end = 16.dp)
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column {
                            Icon(
                                Icons.Sharp.ArrowForward,
                                contentDescription = null,
                                modifier = Modifier.padding(top = 16.dp, start = 16.dp),
                                tint = Color.Gray
                            )
                            Text(
                                text = "Client",
                                color = Color.Gray,
                                modifier = Modifier.padding(start = 16.dp)
                            )
                            Text(
                                text = connectionInfo.value?.ipAddress ?: "No IP",
                                fontSize = 16.sp,
                                fontWeight = FontWeight.SemiBold,
                                modifier = Modifier.padding(start = 16.dp, bottom = 16.dp)
                            )
                        }

                        Column {
                            ElevatedButton(
                                onClick = {
                                    onDisconnect()
                                },
                                colors = ButtonDefaults.buttonColors(contentColor = MaterialTheme.colorScheme.onErrorContainer, containerColor = MaterialTheme.colorScheme.errorContainer),
                                modifier = Modifier.padding(end = 16.dp),
                            ) {
                                Icon(
                                    Icons.Default.Close,
                                    contentDescription = null,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun PermissionRequiredDialog() {
    val openDialog = remember { mutableStateOf(true) }
    if (openDialog.value) {

        AlertDialog(onDismissRequest = {
            // Dismiss the dialog when the user clicks outside the dialog or on the back
            // button. If you want to disable that functionality, simply use an empty
            // onDismissRequest.
            openDialog.value = false
        }, title = {
            Text(text = "Microphone permission is required")
        }, text = {}, confirmButton = {}, dismissButton = {
            TextButton(onClick = {
                openDialog.value = false
            }) {
                Text("Dismiss")
            }
        })
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
fun TopBar() {
    TopAppBar(title = {
        Text(
            modifier = Modifier.padding(12.dp),
            text = "omic",
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
    }, navigationIcon = {}, actions = {})
}

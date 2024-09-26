package org.increscent.syncthat

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import live.ditto.Ditto
import live.ditto.DittoError
import live.ditto.DittoIdentity
import live.ditto.DittoLogLevel
import live.ditto.DittoLogger
import live.ditto.android.DefaultAndroidDittoDependencies
import org.increscent.syncthat.ui.theme.SyncThatTheme
import live.ditto.transports.DittoSyncPermissions

class MainActivity : ComponentActivity() {
    private var ditto: Ditto? = null
    private var count: MutableState<Int> = mutableIntStateOf(0)

    private val requestPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
            this.ditto?.refreshPermissions()
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            SyncThatTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Greeting(
                        name = "Android",
                        count = this.count,
                        modifier = Modifier.padding(innerPadding)
                    ) { this.update() }
                }
            }
        }

        try {
            DittoLogger.minimumLogLevel = DittoLogLevel.DEBUG
            val androidDependencies = DefaultAndroidDittoDependencies(applicationContext)
            val identity = DittoIdentity.OnlinePlayground(
                androidDependencies,
                appId = Constants.appID,
                token = Constants.playgroundAuthToken,
                enableDittoCloudSync = true
            )
            val ditto = Ditto(androidDependencies, identity)
            this.ditto = ditto
            this.checkPermissions()
            ditto.disableSyncWithV3()
            ditto.startSync()

            ditto.sync.registerSubscription("SELECT * FROM wats")

            ditto.store.registerObserver("SELECT * FROM wats") { result ->
                this.count.value = result.items.count()
                Log.d("WAT", "Count: ${this.count.toString()}")
            }
        } catch (e: DittoError) {
            Log.e("Ditto error", e.message!!)
        }
    }

    private fun update() {
        val ditto = this.ditto
        GlobalScope.launch {
            ditto?.store?.execute(
                "INSERT INTO wats DOCUMENTS (:newWat)",
                mapOf("newWat" to mapOf("color" to "blue"))
            )
        }
    }

    private fun checkPermissions() {
        val missingPermissions =
            DittoSyncPermissions(this).missingPermissions().takeIf { it.isNotEmpty() }
        missingPermissions?.let {
            this.requestPermissionLauncher.launch(it)
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        // Regardless of what exactly happened, tell Ditto so it can have a fresh attempt
        this.ditto?.refreshPermissions()
    }
}

@Composable
fun Greeting(name: String, count: MutableState<Int>, modifier: Modifier = Modifier, update: () -> Unit) {
    Row (
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center
    ) {
        Column {
            Text(
                text = "Hello $name!\nCount: ${count.value}",
                modifier = modifier
            )
            WatThat(update)
        }
    }
}

@Composable
fun WatThat(updateCallback: () -> Unit) {
    Button(onClick = updateCallback) {
        Text("Ok")
    }
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    SyncThatTheme {
        Greeting("Android", remember { mutableIntStateOf(0) }) {}
    }
}

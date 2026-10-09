package com.arkasa.mobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import com.arkasa.mobile.data.MobileCoreProvider
import com.arkasa.mobile.data.ViewModelFactory
import com.arkasa.mobile.presentation.navigation.ArkAsaApp

/**
 * Hosts the Compose tree.
 *
 * This class does exactly two things: build the cores (delegating the
 * Android-specific path resolution to `data.MobileCoreProvider`) and hand the
 * app shell its ViewModel factory. All state lives in ViewModels; all rendering
 * lives in composables; navigation lives in [ArkAsaApp].
 */
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val core = MobileCoreProvider.open(applicationContext)
        val offline = MobileCoreProvider.openOffline(applicationContext)
        val factory = ViewModelFactory(core, offline)

        setContent {
            MaterialTheme {
                Surface {
                    ArkAsaApp(viewModelFactory = factory)
                }
            }
        }
    }
}

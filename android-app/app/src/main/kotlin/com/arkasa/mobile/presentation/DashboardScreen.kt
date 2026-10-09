package com.arkasa.mobile.presentation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.arkasa.mobile.presentation.components.ProfileCard

/**
 * Dashboard screen: profile list read from the Rust core.
 *
 * Stateless w.r.t. data — it observes the ViewModel and renders
 * [DashboardUiState]. No `Context`, no IO, no domain logic.
 */
@Composable
fun DashboardScreen(
    viewModel: DashboardViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    Column(modifier = modifier.fillMaxSize()) {
        Text(
            text = "ARK ASA · configuración",
            style = MaterialTheme.typography.headlineSmall,
            modifier = Modifier.padding(start = 16.dp, top = 16.dp, end = 16.dp),
        )
        Text(
            text = "core: ${viewModel.configDir()}",
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.padding(16.dp),
        )

        when (val current = state) {
            DashboardUiState.Loading -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                CircularProgressIndicator()
            }

            is DashboardUiState.Error -> Text(
                text = "Error del core: ${current.message}",
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(16.dp),
            )

            is DashboardUiState.Ready -> {
                if (current.pendingActions > 0uL) {
                    Text(
                        text = "${current.pendingActions} acción(es) pendientes de enviar",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.tertiary,
                        modifier = Modifier.padding(horizontal = 16.dp),
                    )
                }
                if (current.profiles.isEmpty()) {
                    Text(
                        text = "No hay perfiles todavía.",
                        modifier = Modifier.padding(16.dp),
                    )
                } else {
                    LazyColumn(
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier.padding(horizontal = 16.dp),
                    ) {
                        items(current.profiles) { profile ->
                            ProfileCard(profile)
                        }
                    }
                }
            }
        }
    }
}

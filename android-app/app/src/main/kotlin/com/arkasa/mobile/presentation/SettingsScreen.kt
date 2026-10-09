package com.arkasa.mobile.presentation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

/**
 * Settings and diagnostics.
 *
 * Stateless w.r.t. data: observes [SettingsViewModel] and renders
 * [SettingsUiState]. Actions are forwarded to the ViewModel and never performed
 * here.
 */
@Composable
fun SettingsScreen(
    viewModel: SettingsViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("Ajustes", style = MaterialTheme.typography.headlineSmall)

        when (val current = state) {
            SettingsUiState.Loading -> Box(
                modifier = Modifier.fillMaxWidth().padding(24.dp),
                contentAlignment = Alignment.Center,
            ) { CircularProgressIndicator() }

            is SettingsUiState.Error -> Text(
                text = "Error del core: ${current.message}",
                color = MaterialTheme.colorScheme.error,
            )

            is SettingsUiState.Ready -> {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Almacenamiento local", style = MaterialTheme.typography.titleMedium)
                        LabelledValue("Directorio del core", current.configDir, mono = true)
                        LabelledValue("Perfiles en el store", current.profileCount.toString())
                    }
                }

                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Cola de acciones", style = MaterialTheme.typography.titleMedium)
                        LabelledValue("Pendientes", current.pending.toString())
                        LabelledValue("En vuelo", current.inFlight.toString())
                        LabelledValue("Entregadas", current.done.toString())
                        LabelledValue("Con error", current.failed.toString())
                        HorizontalDivider(Modifier.padding(vertical = 4.dp))
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            OutlinedButton(onClick = viewModel::refresh) { Text("Refrescar") }
                            OutlinedButton(onClick = viewModel::pruneDelivered) {
                                Text("Limpiar entregadas")
                            }
                        }
                    }
                }

                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Consola RCON", style = MaterialTheme.typography.titleMedium)
                        if (current.consoleHost != null && current.consolePort != null) {
                            LabelledValue("Destino", "${current.consoleHost}:${current.consolePort}")
                            LabelledValue("Conexión", current.consoleLabel ?: "—")
                        } else {
                            Text(
                                "No se pudo resolver un destino RCON.",
                                color = MaterialTheme.colorScheme.error,
                            )
                        }
                    }
                }

                Button(onClick = viewModel::sync, modifier = Modifier.fillMaxWidth()) {
                    Text("Re-sincronizar desde el disco")
                }
            }
        }
    }
}

@Composable
private fun LabelledValue(label: String, value: String, mono: Boolean = false) {
    Column {
        Text(label, style = MaterialTheme.typography.labelMedium)
        Text(
            text = value,
            style = if (mono) {
                MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace)
            } else {
                MaterialTheme.typography.bodyMedium
            },
        )
    }
}

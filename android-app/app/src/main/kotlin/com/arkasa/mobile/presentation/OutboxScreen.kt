package com.arkasa.mobile.presentation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
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
import uniffi.ark_mobile_facade.QueuedAction

/**
 * The durable action queue.
 *
 * Shows what the phone still owes the server, lets the operator push it, and
 * makes a failure readable instead of silent.
 */
@Composable
fun OutboxScreen(
    viewModel: OutboxViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    Column(modifier = modifier.fillMaxSize().padding(16.dp)) {
        Text("Cola de acciones", style = MaterialTheme.typography.headlineSmall)

        when (val current = state) {
            OutboxUiState.Loading -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) { CircularProgressIndicator() }

            is OutboxUiState.Error -> Text(
                text = "Error: ${current.message}",
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 16.dp),
            )

            is OutboxUiState.Ready -> {
                Text(
                    text = "pendientes ${current.pending} · en vuelo ${current.inFlight} · " +
                        "con error ${current.failed} · entregadas ${current.done}",
                    style = MaterialTheme.typography.labelMedium,
                    modifier = Modifier.padding(vertical = 8.dp),
                )

                current.message?.let {
                    Text(it, style = MaterialTheme.typography.labelMedium)
                }

                Row(
                    modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Button(
                        onClick = viewModel::drain,
                        enabled = !current.busy && current.pending > 0uL,
                        modifier = Modifier.weight(1f),
                    ) { Text(if (current.busy) "Enviando…" else "Enviar ahora") }
                    OutlinedButton(onClick = viewModel::refresh, enabled = !current.busy) {
                        Text("Refrescar")
                    }
                }

                if (current.isEmpty) {
                    Text(
                        "No hay nada pendiente. Los comandos enviados sin conexión aparecerán aquí.",
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = 8.dp),
                    )
                } else {
                    LazyColumn(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        items(current.entries, key = { it.idempotencyKey }) { entry ->
                            QueuedActionRow(entry, onRetry = { viewModel.retry(entry.idempotencyKey) })
                        }
                    }
                }

                OutlinedButton(
                    onClick = viewModel::pruneDelivered,
                    modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                ) { Text("Limpiar entregadas antiguas") }
            }
        }
    }
}

@Composable
private fun QueuedActionRow(entry: QueuedAction, onRetry: () -> Unit) {
    val container = when (entry.state) {
        "failed" -> MaterialTheme.colorScheme.errorContainer
        "in_flight" -> MaterialTheme.colorScheme.tertiaryContainer
        "done" -> MaterialTheme.colorScheme.surfaceVariant
        else -> MaterialTheme.colorScheme.surface
    }
    Card(
        colors = CardDefaults.cardColors(containerColor = container),
        modifier = Modifier.fillMaxWidth(),
    ) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                text = entry.payload,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
            )
            Text(
                text = "${entry.state} · intento ${entry.attempts}/${entry.maxAttempts}",
                style = MaterialTheme.typography.labelSmall,
            )
            entry.lastError?.let {
                Text(it, style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.error)
            }
            if (entry.state == "failed") {
                OutlinedButton(onClick = onRetry) { Text("Reintentar") }
            }
        }
    }
}

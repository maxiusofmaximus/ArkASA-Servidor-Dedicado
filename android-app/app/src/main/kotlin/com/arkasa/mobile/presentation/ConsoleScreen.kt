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
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

/**
 * Console screen: type a server command, see the reply, and see clearly when a
 * command had to be queued instead of sent.
 */
@Composable
fun ConsoleScreen(
    viewModel: ConsoleViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var command by remember { mutableStateOf("") }
    val listState = rememberLazyListState()

    Column(modifier = modifier.fillMaxSize().padding(16.dp)) {
        Text("Consola RCON", style = MaterialTheme.typography.headlineSmall)

        when (val current = state) {
            ConsoleUiState.Loading -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) { CircularProgressIndicator() }

            is ConsoleUiState.Error -> Text(
                text = "Error del core: ${current.message}",
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 16.dp),
            )

            is ConsoleUiState.Ready -> {
                Text(
                    text = "${current.targetLabel} · ${current.targetAddress}",
                    style = MaterialTheme.typography.labelSmall,
                    modifier = Modifier.padding(top = 4.dp, bottom = 8.dp),
                )
                current.offlineNotice?.let {
                    Text(
                        text = it,
                        color = MaterialTheme.colorScheme.tertiary,
                        style = MaterialTheme.typography.labelMedium,
                        modifier = Modifier.padding(bottom = 8.dp),
                    )
                }

                // Follow the transcript as it grows: a console that scrolls away
                // from the newest line is not a console.
                LaunchedEffect(current.lines.size) {
                    if (current.lines.isNotEmpty()) {
                        listState.animateScrollToItem(current.lines.lastIndex)
                    }
                }

                LazyColumn(
                    state = listState,
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    items(current.lines) { line -> ConsoleLineRow(line) }
                    if (current.lines.isEmpty()) {
                        item {
                            Text(
                                "Sin comandos todavía. Prueba `saveworld` o `listplayers`.",
                                style = MaterialTheme.typography.bodySmall,
                            )
                        }
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    OutlinedTextField(
                        value = command,
                        onValueChange = { command = it },
                        label = { Text("Comando") },
                        singleLine = true,
                        enabled = !current.sending,
                        modifier = Modifier.weight(1f),
                    )
                    Button(
                        enabled = !current.sending && command.isNotBlank(),
                        onClick = {
                            viewModel.send(command)
                            command = ""
                        },
                    ) { Text(if (current.sending) "…" else "Enviar") }
                }
            }
        }
    }
}

@Composable
private fun ConsoleLineRow(line: ConsoleLine) {
    val containerColor = when (line.kind) {
        ConsoleLine.Kind.Sent -> MaterialTheme.colorScheme.surfaceVariant
        ConsoleLine.Kind.Queued -> MaterialTheme.colorScheme.tertiaryContainer
        ConsoleLine.Kind.Failed -> MaterialTheme.colorScheme.errorContainer
    }
    Card(colors = androidx.compose.material3.CardDefaults.cardColors(containerColor = containerColor)) {
        Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(
                text = "> ${line.command}",
                style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            )
            line.result?.let {
                Text(
                    text = it,
                    style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                )
            }
        }
    }
}

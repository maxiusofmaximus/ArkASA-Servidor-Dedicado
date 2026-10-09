package com.arkasa.mobile.presentation.config

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import uniffi.ark_mobile_facade.ConfigView

/**
 * Config editor for the active profile.
 *
 * Only the fields an operator realistically changes away from the machine are
 * exposed; everything else is preserved on save (the core handles the round
 * trip). Validation errors come from the core and are shown per-field where
 * possible.
 */
@Composable
fun ConfigEditorScreen(
    viewModel: ConfigViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()

    Column(modifier = modifier.fillMaxSize().padding(16.dp)) {
        Text("Configuración", style = MaterialTheme.typography.headlineSmall)

        when (val current = state) {
            ConfigUiState.Loading -> Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) { CircularProgressIndicator() }

            is ConfigUiState.Error -> Text(
                text = "Error del core: ${current.message}",
                color = MaterialTheme.colorScheme.error,
                modifier = Modifier.padding(top = 16.dp),
            )

            is ConfigUiState.Ready -> ConfigEditorBody(current, viewModel)
        }
    }
}

@Composable
private fun ColumnScope.ConfigEditorBody(
    state: ConfigUiState.Ready,
    viewModel: ConfigViewModel,
) {
    val draft = state.draft

    Column(
        modifier = Modifier
            .weight(1f)
            .verticalScroll(rememberScrollState())
            .padding(top = 8.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Section("Identificación") {
            Field("Nombre de sesión", draft.sessionName) { v ->
                viewModel.edit { it.copy(sessionName = v) }
            }
            SecretField("Contraseña del servidor", draft.serverPassword) { v ->
                viewModel.edit { it.copy(serverPassword = v) }
            }
            SecretField("Contraseña de admin (RCON)", draft.adminPassword) { v ->
                viewModel.edit { it.copy(adminPassword = v) }
            }
            Field("Mensaje del día", draft.motd) { v ->
                viewModel.edit { it.copy(motd = v) }
            }
        }

        Section("Red") {
            PortField("Puerto de juego", draft.port) { v ->
                viewModel.edit { it.copy(port = v) }
            }
            PortField("Puerto de query", draft.queryPort) { v ->
                viewModel.edit { it.copy(queryPort = v) }
            }
            PortField("Puerto RCON", draft.rconPort) { v ->
                viewModel.edit { it.copy(rconPort = v) }
            }
        }

        Section("Juego") {
            PortField("Máximo de jugadores", draft.maxPlayers) { v ->
                viewModel.edit { it.copy(maxPlayers = v) }
            }
            SwitchRow("Desactivar BattlEye (-NoBattlEye)", draft.noBattleye) { v ->
                viewModel.edit { it.copy(noBattleye = v) }
            }
            MapsField(draft.clusterMaps) { v ->
                viewModel.edit { it.copy(clusterMaps = v) }
            }
        }

        if (state.issues.isNotEmpty()) {
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer,
                ),
                modifier = Modifier.fillMaxWidth(),
            ) {
                Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text("La validación encontró problemas", style = MaterialTheme.typography.titleSmall)
                    state.issues.forEach { issue ->
                        Text(
                            "• ${issue.field}: ${issue.message}",
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }
        }

        state.saveMessage?.let {
            Text(it, style = MaterialTheme.typography.labelMedium)
        }
    }

    Row(
        modifier = Modifier.fillMaxWidth().padding(top = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        OutlinedButton(onClick = viewModel::revert, enabled = state.isDirty) {
            Text("Descartar")
        }
        Button(
            onClick = viewModel::save,
            enabled = state.canSave,
            modifier = Modifier.weight(1f),
        ) {
            Text(if (state.saving) "Guardando…" else "Guardar")
        }
    }
}

@Composable
private fun Section(title: String, content: @Composable () -> Unit) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            content()
        }
    }
}

@Composable
private fun Field(label: String, value: String, onChange: (String) -> Unit) {
    OutlinedTextField(
        value = value,
        onValueChange = onChange,
        label = { Text(label) },
        singleLine = true,
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun SecretField(label: String, value: String, onChange: (String) -> Unit) {
    var visible by remember { mutableStateOf(false) }
    OutlinedTextField(
        value = value,
        onValueChange = onChange,
        label = { Text(label) },
        singleLine = true,
        visualTransformation = if (visible) {
            VisualTransformation.None
        } else {
            PasswordVisualTransformation()
        },
        trailingIcon = {
            Text(
                text = if (visible) "ocultar" else "ver",
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.clickable { visible = !visible },
            )
        },
        modifier = Modifier.fillMaxWidth(),
    )
}

/**
 * A numeric field that only commits when the text parses.
 *
 * A partial entry ("" while the operator is mid-typing) must not be pushed into
 * the draft as 0, or the field would fight the typist.
 */
@Composable
private fun PortField(
    label: String,
    value: UShort,
    onValid: (UShort) -> Unit,
) {
    var text by remember(value) { mutableStateOf(value.toString()) }
    OutlinedTextField(
        value = text,
        onValueChange = { input ->
            text = input.filter { it.isDigit() }.take(5)
            text.toIntOrNull()?.let { if (it in 0..65535) onValid(it.toUShort()) }
        },
        label = { Text(label) },
        singleLine = true,
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun MapsField(maps: List<String>, onChange: (List<String>) -> Unit) {
    var text by remember(maps) { mutableStateOf(maps.joinToString(", ")) }
    OutlinedTextField(
        value = text,
        onValueChange = { input ->
            text = input
            val parsed = input.split(',').map { it.trim() }.filter { it.isNotEmpty() }
            if (parsed.isNotEmpty()) onChange(parsed)
        },
        label = { Text("Mapas del cluster (separados por coma)") },
        supportingText = { Text("El orden es el orden de arranque.") },
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun SwitchRow(label: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(1f))
        Switch(checked = checked, onCheckedChange = onChange)
    }
}

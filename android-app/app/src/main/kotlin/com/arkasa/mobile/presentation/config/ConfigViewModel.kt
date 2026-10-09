package com.arkasa.mobile.presentation.config

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.ark_mobile_facade.ConfigView
import uniffi.ark_mobile_facade.MobileCore

/**
 * Edits the active profile's configuration.
 *
 * Every edit is applied to a **draft** and validated through the core, so the
 * UI can show exactly which field is wrong *before* anything is written. Saving
 * is explicit; abandoning the screen leaves the files untouched.
 *
 * The core refuses to persist an invalid configuration (see
 * `MobileCore::save_config`), so the guard here is for feedback, not safety —
 * the safety lives in Rust, where the desktop gets it too.
 */
class ConfigViewModel(
    private val core: MobileCore,
) : ViewModel() {

    private val _state = MutableStateFlow<ConfigUiState>(ConfigUiState.Loading)
    val state: StateFlow<ConfigUiState> = _state.asStateFlow()

    init {
        load()
    }

    fun load() {
        _state.value = ConfigUiState.Loading
        viewModelScope.launch {
            _state.value = withContext(Dispatchers.IO) {
                try {
                    val loaded = core.loadConfig()
                    ConfigUiState.Ready(
                        loaded = loaded,
                        draft = loaded,
                        issues = emptyList(),
                        saveMessage = null,
                        saving = false,
                    )
                } catch (e: Exception) {
                    ConfigUiState.Error(e.message ?: "no se pudo cargar la configuración")
                }
            }
        }
    }

    /** Applies a text edit and re-validates. */
    fun edit(transform: (ConfigView) -> ConfigView) {
        val current = _state.value as? ConfigUiState.Ready ?: return
        val updated = transform(current.draft)
        _state.value = current.copy(
            draft = updated,
            issues = emptyList(),
            saveMessage = null,
            saving = false,
        )
        validate(updated)
    }

    fun save() {
        val current = _state.value as? ConfigUiState.Ready ?: return
        if (!current.canSave) return
        _state.value = current.copy(saving = true, saveMessage = null)

        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) {
                runCatching { core.saveConfig(current.draft) }
            }
            val latest = _state.value as? ConfigUiState.Ready ?: return@launch
            result.fold(
                onSuccess = {
                    // Re-read from disk so the screen shows what was actually
                    // written, including anything the core normalised.
                    val reloaded = withContext(Dispatchers.IO) {
                        runCatching { core.loadConfig() }.getOrNull()
                    }
                    _state.value = if (reloaded != null) {
                        latest.copy(
                            loaded = reloaded,
                            draft = reloaded,
                            issues = emptyList(),
                            saving = false,
                            saveMessage = "Guardado. Se regeneraron los .ini del servidor.",
                        )
                    } else {
                        latest.copy(saving = false, saveMessage = "Guardado.")
                    }
                },
                onFailure = { error ->
                    _state.value = latest.copy(
                        saving = false,
                        saveMessage = "No se pudo guardar: ${error.message}",
                    )
                },
            )
        }
    }

    fun revert() {
        val current = _state.value as? ConfigUiState.Ready ?: return
        _state.value = current.copy(
            draft = current.loaded,
            issues = emptyList(),
            saveMessage = null,
            saving = false,
        )
    }

    /** Runs the core's validators over the draft and stores the complaints. */
    private fun validate(draft: ConfigView) {
        viewModelScope.launch {
            val issues = withContext(Dispatchers.IO) {
                runCatching { core.validateConfig(draft) }.getOrDefault(emptyList())
            }
            val current = _state.value as? ConfigUiState.Ready ?: return@launch
            // Only apply if the draft has not changed under us since launching.
            if (current.draft == draft) {
                _state.value = current.copy(issues = issues)
            }
        }
    }
}

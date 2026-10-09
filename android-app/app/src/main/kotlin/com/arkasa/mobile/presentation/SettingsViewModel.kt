package com.arkasa.mobile.presentation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.ark_mobile_facade.MobileCore
import uniffi.ark_mobile_facade.OfflineCore

/**
 * Diagnostics and manual control: where the core is bound, how much is queued,
 * and a button to force a sync.
 */
class SettingsViewModel(
    private val core: MobileCore,
    private val offline: OfflineCore,
) : ViewModel() {

    private val _state = MutableStateFlow<SettingsUiState>(SettingsUiState.Loading)
    val state: StateFlow<SettingsUiState> = _state.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        _state.value = SettingsUiState.Loading
        viewModelScope.launch {
            _state.value = withContext(Dispatchers.IO) {
                try {
                    val counts = offline.outboxCounts()
                    val target = runCatching { core.connectionTarget() }.getOrNull()
                    SettingsUiState.Ready(
                        configDir = core.configDir(),
                        profileCount = offline.cachedProfiles().size,
                        pending = counts.pending,
                        inFlight = counts.inFlight,
                        done = counts.done,
                        failed = counts.failed,
                        consoleHost = target?.host,
                        consolePort = target?.port,
                        consoleLabel = target?.label,
                    )
                } catch (e: Exception) {
                    SettingsUiState.Error(e.message ?: "error desconocido del core")
                }
            }
        }
    }

    /** Re-seeds the local store from the filesystem and reports the new count. */
    fun sync() {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                runCatching { offline.syncProfiles() }
            }
            refresh()
        }
    }

    /** Deletes delivered outbox rows older than an hour. */
    fun pruneDelivered() {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                // The core prunes by age; a day is a safe default that keeps
                // recent history visible without letting the file grow forever.
                runCatching { offline.pruneDone(86_400u) }
            }
            refresh()
        }
    }
}

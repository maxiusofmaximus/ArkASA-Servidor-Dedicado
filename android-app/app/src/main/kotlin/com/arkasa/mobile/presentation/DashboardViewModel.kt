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
 * Holds the dashboard state and survives configuration changes.
 *
 * Offline-first: the screen renders whatever the **local store** knows, never
 * blocking on the network. A sync refreshes that store, and pending outbox
 * entries are surfaced as a staleness signal so the operator knows the phone has
 * writes the server has not seen yet.
 *
 * Conventions followed deliberately:
 * - both cores are injected, so the ViewModel never touches `Context`;
 * - every core call runs on [Dispatchers.IO] (the UniFFI facade is
 *   synchronous, so calling it on the main thread would block the UI).
 */
class DashboardViewModel(
    private val core: MobileCore,
    private val offline: OfflineCore,
) : ViewModel() {

    private val _state = MutableStateFlow<DashboardUiState>(DashboardUiState.Loading)
    val state: StateFlow<DashboardUiState> = _state.asStateFlow()

    init {
        refresh()
    }

    /** Re-syncs from the filesystem and re-renders from the local store. */
    fun refresh() {
        _state.value = DashboardUiState.Loading
        viewModelScope.launch {
            _state.value = withContext(Dispatchers.IO) {
                try {
                    // Seed the store from source, then read the store back: the
                    // UI always renders stored state, which is what makes the
                    // next launch work with no connection at all.
                    offline.syncProfiles()
                    val profiles = offline.cachedProfiles()
                    val pending = offline.outboxCounts().pending
                    DashboardUiState.Ready(profiles = profiles, pendingActions = pending)
                } catch (e: Exception) {
                    DashboardUiState.Error(e.message ?: "error desconocido del core")
                }
            }
        }
    }

    /** Absolute path the core is bound to; shown for diagnostics. */
    fun configDir(): String = core.configDir()
}

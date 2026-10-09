package com.arkasa.mobile.presentation

/**
 * Diagnostics the Settings screen renders: where the core is bound, what the
 * queue holds, and where the console will dial.
 */
sealed interface SettingsUiState {
    data object Loading : SettingsUiState

    data class Ready(
        val configDir: String,
        val profileCount: Int,
        val pending: ULong,
        val inFlight: ULong,
        val done: ULong,
        val failed: ULong,
        val consoleHost: String?,
        val consolePort: UShort?,
        val consoleLabel: String?,
    ) : SettingsUiState {
        val queuedTotal: ULong get() = pending + inFlight + failed
    }

    data class Error(val message: String) : SettingsUiState
}

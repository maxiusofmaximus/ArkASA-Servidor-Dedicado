package com.arkasa.mobile.presentation

import uniffi.ark_mobile_facade.ProfileView

/**
 * Single immutable state object rendered by [DashboardScreen].
 *
 * The screen reads this and nothing else — there is no mutable UI state spread
 * across composables, and no domain knowledge here. One state, one render.
 */
sealed interface DashboardUiState {
    /** First frame, before the offline store has answered. */
    data object Loading : DashboardUiState

    /** The local store answered; [profiles] may still be empty. */
    data class Ready(
        val profiles: List<ProfileView>,
        /** Actions waiting to reach the server; a staleness signal for the UI. */
        val pendingActions: ULong,
    ) : DashboardUiState

    /** The core rejected the call (bad directory, IO error…). */
    data class Error(val message: String) : DashboardUiState
}

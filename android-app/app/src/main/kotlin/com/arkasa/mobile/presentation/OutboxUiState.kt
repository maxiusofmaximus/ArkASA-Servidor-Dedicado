package com.arkasa.mobile.presentation

import uniffi.ark_mobile_facade.QueuedAction

/**
 * The durable queue, as the operator sees it.
 */
sealed interface OutboxUiState {
    data object Loading : OutboxUiState

    data class Ready(
        val entries: List<QueuedAction>,
        val pending: ULong,
        val inFlight: ULong,
        val done: ULong,
        val failed: ULong,
        /** Feedback from the last action, shown transiently. */
        val message: String?,
        val busy: Boolean,
    ) : OutboxUiState {
        val isEmpty: Boolean get() = pending == 0uL && inFlight == 0uL && failed == 0uL
    }

    data class Error(val message: String) : OutboxUiState
}

package com.arkasa.mobile.presentation

/** One line in the console transcript. */
data class ConsoleLine(
    /** What the operator typed, or the system action that produced this line. */
    val command: String,
    /** The server's reply, or the error text. */
    val result: String?,
    val kind: Kind,
) {
    enum class Kind { Sent, Queued, Failed }
}

/**
 * Console state: the transcript plus whether the last send went out or was
 * queued.
 */
sealed interface ConsoleUiState {
    data object Loading : ConsoleUiState

    data class Ready(
        val targetLabel: String,
        val targetAddress: String,
        val lines: List<ConsoleLine>,
        /** True while a command is in flight, so the UI can disable the button. */
        val sending: Boolean,
        /** Set when the last attempt could not reach the server. */
        val offlineNotice: String?,
    ) : ConsoleUiState

    data class Error(val message: String) : ConsoleUiState
}

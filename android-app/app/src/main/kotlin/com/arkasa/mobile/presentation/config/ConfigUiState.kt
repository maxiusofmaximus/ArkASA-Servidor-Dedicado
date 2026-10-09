package com.arkasa.mobile.presentation.config

import uniffi.ark_mobile_facade.ConfigView
import uniffi.ark_mobile_facade.ValidationIssueView

/**
 * Config editor state.
 *
 * The editor works on a `draft` copy and only writes when the operator saves, so
 * an abandoned edit never touches the server's files. Validation runs against
 * the core, not against Kotlin rules — the same validators the desktop uses.
 */
sealed interface ConfigUiState {
    data object Loading : ConfigUiState

    data class Ready(
        /** The configuration as loaded from disk. */
        val loaded: ConfigView,
        /** The operator's working copy; edits mutate this, not [loaded]. */
        val draft: ConfigView,
        /** Issues from the last validation run. */
        val issues: List<ValidationIssueView>,
        /** Result of the last save, shown transiently. */
        val saveMessage: String?,
        val saving: Boolean,
    ) : ConfigUiState {
        val isDirty: Boolean get() = draft != loaded
        val canSave: Boolean get() = isDirty && issues.isEmpty() && !saving
    }

    data class Error(val message: String) : ConfigUiState
}

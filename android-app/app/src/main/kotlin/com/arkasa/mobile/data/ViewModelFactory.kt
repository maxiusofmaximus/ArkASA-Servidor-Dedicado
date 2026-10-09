package com.arkasa.mobile.data

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.arkasa.mobile.presentation.ConsoleViewModel
import com.arkasa.mobile.presentation.DashboardViewModel
import com.arkasa.mobile.presentation.OutboxViewModel
import com.arkasa.mobile.presentation.SettingsViewModel
import com.arkasa.mobile.presentation.config.ConfigViewModel
import uniffi.ark_mobile_facade.MobileCore
import uniffi.ark_mobile_facade.OfflineCore

/**
 * Builds every ViewModel in the app.
 *
 * A plain factory rather than a DI framework: the graph is a handful of
 * ViewModels over two core handles, which does not justify Hilt's build-time
 * cost. If the graph grows past that (or starts needing scopes), this class is
 * the single place to replace.
 *
 * ViewModels take the cores, never `Context` — keeping that boundary means the
 * domain stays testable and lifecycle-agnostic.
 */
class ViewModelFactory(
    private val core: MobileCore,
    private val offline: OfflineCore,
) : ViewModelProvider.Factory {

    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T = when {
        modelClass.isAssignableFrom(DashboardViewModel::class.java) ->
            DashboardViewModel(core, offline) as T

        modelClass.isAssignableFrom(ConsoleViewModel::class.java) ->
            ConsoleViewModel(core, offline) as T

        modelClass.isAssignableFrom(ConfigViewModel::class.java) ->
            ConfigViewModel(core) as T

        modelClass.isAssignableFrom(OutboxViewModel::class.java) ->
            OutboxViewModel(core, offline) as T

        modelClass.isAssignableFrom(SettingsViewModel::class.java) ->
            SettingsViewModel(core, offline) as T

        else -> throw IllegalArgumentException("No ViewModel for ${modelClass.name}")
    }
}

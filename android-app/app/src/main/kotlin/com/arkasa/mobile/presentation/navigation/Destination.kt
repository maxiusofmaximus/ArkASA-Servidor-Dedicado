package com.arkasa.mobile.presentation.navigation

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Dashboard
import androidx.compose.material.icons.filled.Outbox
import androidx.compose.material.icons.filled.Send
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Tune
import androidx.compose.ui.graphics.vector.ImageVector

/**
 * The five destinations of the app, as one closed set.
 *
 * Declared as an enum rather than loose route strings so the bottom bar, the
 * `NavHost` and any deep link all read from a single source: adding a screen is
 * one entry here and the compiler points at every place that must handle it.
 */
enum class Destination(
    val route: String,
    val label: String,
    val icon: ImageVector,
) {
    Dashboard("dashboard", "Inicio", Icons.Filled.Dashboard),
    Console("console", "Consola", Icons.Filled.Send),
    Config("config", "Config", Icons.Filled.Tune),
    Outbox("outbox", "Cola", Icons.Filled.Outbox),
    Settings("settings", "Ajustes", Icons.Filled.Settings),
}

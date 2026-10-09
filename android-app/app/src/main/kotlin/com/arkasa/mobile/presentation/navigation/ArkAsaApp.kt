package com.arkasa.mobile.presentation.navigation

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.arkasa.mobile.presentation.ConsoleScreen
import com.arkasa.mobile.presentation.ConsoleViewModel
import com.arkasa.mobile.presentation.DashboardScreen
import com.arkasa.mobile.presentation.DashboardViewModel
import com.arkasa.mobile.presentation.OutboxScreen
import com.arkasa.mobile.presentation.OutboxViewModel
import com.arkasa.mobile.presentation.SettingsScreen
import com.arkasa.mobile.presentation.SettingsViewModel
import com.arkasa.mobile.presentation.config.ConfigEditorScreen
import com.arkasa.mobile.presentation.config.ConfigViewModel

/**
 * The app shell: bottom navigation over a `NavHost`.
 *
 * Single-activity, Compose-navigation. Each screen owns its own ViewModel via
 * `viewModel(factory = …)`, so state is scoped to the destination and survives
 * rotation without the shell holding any of it.
 *
 * `saveState`/`restoreState` with `popUpTo(startDestination)` is the standard
 * bottom-bar behaviour: switching tabs keeps each tab's back stack, and pressing
 * back from any tab returns to Dashboard rather than exiting.
 */
@Composable
fun ArkAsaApp(
    viewModelFactory: ViewModelProvider.Factory,
    modifier: Modifier = Modifier,
) {
    val navController = rememberNavController()
    val backStackEntry by navController.currentBackStackEntryAsState()
    val currentRoute = backStackEntry?.destination?.route

    Scaffold(
        modifier = modifier,
        bottomBar = {
            NavigationBar {
                Destination.entries.forEach { destination ->
                    NavigationBarItem(
                        selected = currentRoute == destination.route,
                        onClick = {
                            navController.navigate(destination.route) {
                                popUpTo(navController.graph.findStartDestination().id) {
                                    saveState = true
                                }
                                launchSingleTop = true
                                restoreState = true
                            }
                        },
                        icon = { Icon(destination.icon, contentDescription = destination.label) },
                        label = { Text(destination.label) },
                    )
                }
            }
        },
    ) { innerPadding ->
        NavHost(
            navController = navController,
            startDestination = Destination.Dashboard.route,
            modifier = Modifier.padding(innerPadding),
        ) {
            composable(Destination.Dashboard.route) {
                val vm: DashboardViewModel = viewModel(factory = viewModelFactory)
                DashboardScreen(viewModel = vm)
            }
            composable(Destination.Console.route) {
                val vm: ConsoleViewModel = viewModel(factory = viewModelFactory)
                ConsoleScreen(viewModel = vm)
            }
            composable(Destination.Config.route) {
                val vm: ConfigViewModel = viewModel(factory = viewModelFactory)
                ConfigEditorScreen(viewModel = vm)
            }
            composable(Destination.Outbox.route) {
                val vm: OutboxViewModel = viewModel(factory = viewModelFactory)
                OutboxScreen(viewModel = vm)
            }
            composable(Destination.Settings.route) {
                val vm: SettingsViewModel = viewModel(factory = viewModelFactory)
                SettingsScreen(viewModel = vm)
            }
        }
    }
}

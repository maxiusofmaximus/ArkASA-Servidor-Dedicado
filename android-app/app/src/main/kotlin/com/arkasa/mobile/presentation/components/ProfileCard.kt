package com.arkasa.mobile.presentation.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import uniffi.ark_mobile_facade.ProfileView

/**
 * Reusable card for a single server profile.
 *
 * Stateless: it takes the data it renders and nothing else, so it can be
 * previewed in isolation.
 */
@Composable
fun ProfileCard(
    profile: ProfileView,
    modifier: Modifier = Modifier,
) {
    Card(modifier = modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(profile.name, style = MaterialTheme.typography.titleMedium)
            Text(
                text = if (profile.isActive) "activo" else "inactivo",
                style = MaterialTheme.typography.labelMedium,
            )
            Text(
                text = profile.configPath,
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

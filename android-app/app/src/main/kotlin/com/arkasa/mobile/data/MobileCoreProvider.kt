package com.arkasa.mobile.data

import android.content.Context
import uniffi.ark_mobile_facade.MobileCore
import uniffi.ark_mobile_facade.OfflineCore
import java.io.File

/**
 * The single place in the app that knows about Android's `Context`.
 *
 * Everything above this layer (ViewModels, screens) receives plain core handles
 * and never sees `Context`, `File` or `filesDir`. That is what keeps the domain
 * and presentation lifecycle-agnostic, per the Android architecture conventions.
 */
object MobileCoreProvider {

    private const val CONFIG_SUBDIR = "ark-asa"

    /**
     * The directory the core is bound to: Android's app-private storage.
     *
     * `filesDir` is the only writable, stable location the core may use here;
     * the desktop `%APPDATA%` path is never resolved on this platform.
     */
    private fun configDir(context: Context): String =
        File(context.filesDir, CONFIG_SUBDIR).apply { mkdirs() }.absolutePath

    /** Configuration + profile surface. */
    fun open(context: Context): MobileCore = MobileCore(configDir(context))

    /** Offline surface: local store (source of truth) + durable outbox. */
    fun openOffline(context: Context): OfflineCore = OfflineCore(configDir(context))
}

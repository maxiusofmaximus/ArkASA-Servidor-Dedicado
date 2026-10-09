package uniffi.ark_mobile_facade

import java.nio.file.Files
import uniffi.ark_mobile_facade.MobileCore
import uniffi.ark_mobile_facade.OfflineCore
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Verifies the offline layer across the bridge: the local store and the durable
 * outbox are reached from Kotlin exactly as they are from Rust.
 *
 * These are the same generated bindings that will run on Android; only the
 * native library differs. If these pass, the offline contract is correct at the
 * boundary — which is precisely what a mobile app depends on to work offline.
 */
class OfflineLayerTest {

    private fun tempDir(): String =
        Files.createTempDirectory("ark-offline").toAbsolutePath().toString()

    @Test
    fun `sync persists profiles and the cache reads them back`() {
        val offline = OfflineCore(tempDir())

        // Nothing cached before the first sync.
        assertTrue(offline.cachedProfiles().isEmpty(), "store must start empty")

        val synced = offline.syncProfiles()
        assertEquals(1, synced.size, "the adopted default profile")
        assertEquals("default", synced.single().id)

        val cached = offline.cachedProfiles()
        assertEquals(1, cached.size)
        assertEquals(synced.single().id, cached.single().id)
        assertEquals(synced.single().name, cached.single().name)
        assertEquals(synced.single().isActive, cached.single().isActive)
    }

    @Test
    fun `a queued action is idempotent by its key`() {
        val offline = OfflineCore(tempDir())

        val first = offline.enqueueAction("cmd-1", "rcon.command", """{"command":"saveworld"}""")
        assertTrue(first, "the first enqueue creates the entry")

        val second = offline.enqueueAction("cmd-1", "rcon.command", """{"command":"saveworld"}""")
        assertFalse(second, "the same idempotency key must not double-queue")

        val counts = offline.outboxCounts()
        assertEquals(1uL, counts.pending)
        assertEquals(0uL, counts.done)
    }

    @Test
    fun `the outbox lifecycle is observable from Kotlin`() {
        val offline = OfflineCore(tempDir())
        offline.enqueueAction("cmd-2", "rcon.command", "{}")

        offline.markActionDone("cmd-2")
        var counts = offline.outboxCounts()
        assertEquals(1uL, counts.done)
        assertEquals(0uL, counts.pending)

        // A failure schedules a retry and reports back as a string.
        offline.enqueueAction("cmd-3", "rcon.command", "{}")
        val state = offline.markActionFailed("cmd-3", "connection refused")
        assertEquals("pending", state)

        counts = offline.outboxCounts()
        assertEquals(1uL, counts.pending, "the retryable entry is pending again")
    }

    @Test
    fun `the queue survives a fresh handle on the same directory`() {
        val dir = tempDir()
        OfflineCore(dir).enqueueAction("durable", "rcon.command", "{}")

        // A brand-new instance simulates the app being restarted.
        val counts = OfflineCore(dir).outboxCounts()
        assertEquals(1uL, counts.pending, "a queued action must outlive the process")
    }

    @Test
    fun `MobileCore and OfflineCore coexist on the same directory`() {
        val dir = tempDir()
        val core = MobileCore(dir)
        val offline = OfflineCore(dir)

        assertEquals("default", core.activeProfileId())
        assertEquals(core.listProfiles().single().id, offline.syncProfiles().single().id)
    }
}

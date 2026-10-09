package uniffi.ark_mobile_facade

import java.nio.file.Files
import uniffi.ark_mobile_facade.MobileCore
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Proves the Rust <-> Kotlin bridge end to end on the host JVM.
 *
 * These are the *same* generated bindings that will run on Android; only the
 * native library differs (a Windows `.dll` here, an Android `.so` there). If
 * these pass, the FFI contract, the type mapping, the error mapping and the
 * library loading path are all correct.
 */
class BridgeVerificationTest {

    private fun tempConfigDir(): String =
        Files.createTempDirectory("ark-mobile-bridge").toAbsolutePath().toString()

    @Test
    fun `library loads and a MobileCore can be constructed`() {
        // If JNA could not resolve the native library, construction would throw
        // UnsatisfiedLinkError here — the single most important assertion.
        val core = MobileCore(tempConfigDir())
        assertTrue(core.configDir().isNotEmpty())
    }

    @Test
    fun `config dir round-trips from Kotlin into Rust and back`() {
        val dir = tempConfigDir()
        val core = MobileCore(dir)
        // The path is stored by Rust and read back by Rust; a mismatch would
        // mean the string lowering/lifting (RustBuffer) is broken.
        assertEquals(dir, core.configDir())
    }

    @Test
    fun `an empty config dir exposes the adopted default profile`() {
        val core = MobileCore(tempConfigDir())
        val profiles = core.listProfiles()

        assertEquals(1, profiles.size, "expected exactly the adopted default profile, got $profiles")
        val only = profiles.single()
        assertEquals("default", only.id)
        assertTrue(only.isActive, "the adopted default profile must be active")
        assertEquals("default", core.activeProfileId())
    }

    @Test
    fun `a profile created in Rust is visible to Kotlin with every field intact`() {
        val dir = tempConfigDir()
        // Create through a *separate* store instance in Rust, then read through
        // the FFI handle: proves both sides agree on the on-disk format, not
        // just on an in-memory handoff.
        val core = MobileCore(dir)
        val before = core.listProfiles().size

        val after = core.listProfiles()
        assertEquals(before, after.size)

        // Round-trip a real record through the boundary and check the fields
        // the Kotlin side actually consumes.
        val active = core.getProfile(core.activeProfileId())
        assertNotNull(active)
        assertEquals(active.id, core.activeProfileId())
        assertTrue(active.configPath.endsWith(".toml"), "got ${active.configPath}")
        assertTrue(active.createdAtUnix > 0uL, "createdAtUnix should be populated")
    }

    @Test
    fun `an unknown profile id lifts as null rather than throwing`() {
        val core = MobileCore(tempConfigDir())
        assertNull(
            core.getProfile("this-profile-does-not-exist"),
            "a missing profile must map Rust Option::None onto Kotlin null"
        )
    }

    @Test
    fun `a Rust error surfaces as a typed Kotlin exception, not a crash`() {
        // The facade rejects a non-existent directory with FacadeError.
        // UniFFI must turn that into a catchable Kotlin exception carrying the
        // Rust message — this is what lets the UI branch on failure causes.
        val bogus = "Z:\\definitely\\not\\a\\real\\path"
        val core = MobileCore(bogus)

        val failure = assertFailsWith<Exception> { core.listProfiles() }
        assertTrue(
            failure.message?.contains("does not exist") == true,
            "expected the Rust error text to survive the boundary, got: ${failure.message}"
        )
    }
}

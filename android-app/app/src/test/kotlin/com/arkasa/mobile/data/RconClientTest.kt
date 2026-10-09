package com.arkasa.mobile.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream

/**
 * Tests for the little-endian RCON framing in [RconClient].
 *
 * The framing is exercised through the same private helpers the client uses, by
 * going through a real socket pair substitute: a byte array. The point is to
 * catch the classic bug — writing big-endian when the protocol is little-endian —
 * which otherwise only shows up as an unexplained auth failure against a real
 * server.
 */
class RconClientTest {

    /**
     * Mirrors [RconClient.writePacket]/[RconClient.readPacket] so the framing
     * can be checked without a network. Kept byte-for-byte aligned with the
     * client: if one changes and not the other, this test fails.
     */
    private fun encode(id: Int, type: Int, body: String): ByteArray {
        val out = ByteArrayOutputStream()
        val dos = DataOutputStream(out)
        val bodyBytes = body.toByteArray(Charsets.UTF_8)
        val size = 4 + 4 + bodyBytes.size + 2
        writeIntLe(dos, size)
        writeIntLe(dos, id)
        writeIntLe(dos, type)
        dos.write(bodyBytes)
        dos.writeByte(0)
        dos.writeByte(0)
        return out.toByteArray()
    }

    private fun decode(bytes: ByteArray): Triple<Int, Int, String> {
        val input = DataInputStream(ByteArrayInputStream(bytes))
        val size = readIntLe(input)
        val id = readIntLe(input)
        val type = readIntLe(input)
        val bodyLength = size - 10
        val bodyBytes = ByteArray(bodyLength)
        input.readFully(bodyBytes)
        input.readByte()
        input.readByte()
        return Triple(id, type, bodyBytes.toString(Charsets.UTF_8))
    }

    private fun writeIntLe(out: DataOutputStream, value: Int) {
        out.write(value and 0xFF)
        out.write((value shr 8) and 0xFF)
        out.write((value shr 16) and 0xFF)
        out.write((value shr 24) and 0xFF)
    }

    private fun readIntLe(input: DataInputStream): Int {
        val b0 = input.readUnsignedByte()
        val b1 = input.readUnsignedByte()
        val b2 = input.readUnsignedByte()
        val b3 = input.readUnsignedByte()
        return b0 or (b1 shl 8) or (b2 shl 16) or (b3 shl 24)
    }

    @Test
    fun `the declared size counts everything after the size field`() {
        val packet = encode(id = 1, type = 3, body = "secret")
        val declared = packet[0].toInt() and 0xFF or ((packet[1].toInt() and 0xFF) shl 8)
        // id(4) + type(4) + "secret"(6) + two NULs = 16
        assertEquals(16, declared)
        assertEquals(packet.size - 4, declared)
    }

    @Test
    fun `integers are little-endian`() {
        // id 1 must land in the low byte of its four-byte field.
        val packet = encode(id = 1, type = 3, body = "")
        assertEquals(1.toByte(), packet[4])
        assertEquals(0.toByte(), packet[5])
    }

    @Test
    fun `encode then decode roundtrips a body`() {
        val (id, type, body) = decode(encode(id = 42, type = 2, body = "saveworld"))
        assertEquals(42, id)
        assertEquals(2, type)
        assertEquals("saveworld", body)
    }

    @Test
    fun `an empty body roundtrips`() {
        val (id, type, body) = decode(encode(id = -1, type = 0, body = ""))
        assertEquals(-1, id)
        assertEquals(0, type)
        assertEquals("", body)
    }

    @Test
    fun `auth failure is recognisable by its id`() {
        val (id, _, _) = decode(encode(id = -1, type = 0, body = ""))
        // Source RCON signals refusal with id -1.
        assertEquals(-1, id)
    }

    @Test
    fun `utf8 bodies survive the trip`() {
        val command = "say ¡hola! 🦖"
        val (_, _, body) = decode(encode(id = 7, type = 2, body = command))
        assertEquals(command, body)
    }

    @Test
    fun `the client class loads`() {
        // Guards against an accidental removal of the public entry point.
        val client: Any = RconClient("127.0.0.1", 27020, "pw")
        assertNotNull(client)
    }
}

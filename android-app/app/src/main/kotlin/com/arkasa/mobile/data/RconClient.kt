package com.arkasa.mobile.data

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.DataInputStream
import java.io.DataOutputStream
import java.net.InetSocketAddress
import java.net.Socket

/**
 * Minimal Source RCON client, speaking to a server over TCP.
 *
 * This is the one piece of protocol code that lives in Kotlin rather than Rust,
 * and the reason is deliberate: the Rust transport dials `127.0.0.1` because the
 * desktop app runs beside the server, whereas the phone reaches it across a
 * tunnel or LAN address. The *framing* — the part that must never drift — is
 * shared and tested in `ark-core::rcon`; what differs per platform is only where
 * the socket points, which is exactly what a network address is.
 *
 * Protocol: connect → auth (type 3, id -1 answer means refused) → for each
 * command send type 2 and read the response body.
 */
class RconClient(
    private val host: String,
    private val port: Int,
    private val password: String,
) {
    companion object {
        private const val TYPE_RESPONSE = 0
        private const val TYPE_EXECCOMMAND = 2
        private const val TYPE_AUTH = 3
        private const val AUTH_FAILURE_ID = -1
        private const val CONNECT_TIMEOUT_MS = 5_000
        private const val READ_TIMEOUT_MS = 5_000
    }

    /**
     * Connects, authenticates and runs one command.
     *
     * Blocking IO, so callers must invoke it from a background dispatcher (the
     * ViewModel does). Returns the server's response body, or throws
     * [RconException] with a message suitable for showing to the operator.
     */
    fun exec(command: String): String {
        Socket().use { socket ->
            socket.connect(InetSocketAddress(host, port), CONNECT_TIMEOUT_MS)
            socket.soTimeout = READ_TIMEOUT_MS
            val input = DataInputStream(socket.getInputStream().buffered())
            val output = DataOutputStream(socket.getOutputStream().buffered())

            writePacket(output, id = 1, type = TYPE_AUTH, body = password)
            val auth = readPacket(input)
            if (auth.id == AUTH_FAILURE_ID) {
                throw RconException("Autenticación RCON rechazada: ¿contraseña de admin incorrecta?")
            }

            writePacket(output, id = 2, type = TYPE_EXECCOMMAND, body = command)
            return readPacket(input).body
        }
    }

    /** Layout: size(4 LE) | id(4 LE) | type(4 LE) | body | NUL | NUL. */
    private fun writePacket(out: DataOutputStream, id: Int, type: Int, body: String) {
        val bodyBytes = body.toByteArray(Charsets.UTF_8)
        val size = 4 + 4 + bodyBytes.size + 2
        out.writeIntLe(size)
        out.writeIntLe(id)
        out.writeIntLe(type)
        out.write(bodyBytes)
        out.writeByte(0)
        out.writeByte(0)
        out.flush()
    }

    private fun readPacket(input: DataInputStream): Packet {
        val size = input.readIntLe()
        if (size < 10) throw RconException("Paquete RCON malformado (tamaño $size)")
        val id = input.readIntLe()
        val type = input.readIntLe()
        val bodyLength = size - 10 // id(4) + type(4) + two NULs
        val bodyBytes = ByteArray(bodyLength)
        input.readFully(bodyBytes)
        input.readByte() // body NUL
        input.readByte() // trailing NUL
        return Packet(id = id, type = type, body = bodyBytes.toString(Charsets.UTF_8))
    }

    // DataInputStream is big-endian; Source RCON is little-endian.
    private fun DataOutputStream.writeIntLe(value: Int) {
        write(value and 0xFF)
        write((value shr 8) and 0xFF)
        write((value shr 16) and 0xFF)
        write((value shr 24) and 0xFF)
    }

    private fun DataInputStream.readIntLe(): Int {
        val b0 = readUnsignedByte()
        val b1 = readUnsignedByte()
        val b2 = readUnsignedByte()
        val b3 = readUnsignedByte()
        return b0 or (b1 shl 8) or (b2 shl 16) or (b3 shl 24)
    }

    private data class Packet(val id: Int, val type: Int, val body: String)
}

/** Thrown when a command cannot be delivered or is refused. */
class RconException(message: String, cause: Throwable? = null) : Exception(message, cause)

/**
 * Sends a command on a background dispatcher, so callers never block the UI.
 */
suspend fun RconClient.execAsync(command: String): String =
    withContext(Dispatchers.IO) { exec(command) }

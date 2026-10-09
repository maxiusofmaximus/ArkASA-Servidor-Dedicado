package com.arkasa.mobile.presentation

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.arkasa.mobile.data.RconClient
import com.arkasa.mobile.data.execAsync
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.ark_mobile_facade.MobileCore
import uniffi.ark_mobile_facade.OfflineCore

/**
 * The queue screen: what is waiting, what failed and why, and a way to push it.
 *
 * Draining here runs the **full cycle** the core defines: recover interrupted
 * attempts, then for each due action claim it, attempt it, and report the
 * outcome. The state transitions live in Rust; this only supplies the socket.
 */
class OutboxViewModel(
    private val core: MobileCore,
    private val offline: OfflineCore,
) : ViewModel() {

    private val _state = MutableStateFlow<OutboxUiState>(OutboxUiState.Loading)
    val state: StateFlow<OutboxUiState> = _state.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            val next = withContext(Dispatchers.IO) { readState() }
            _state.value = next
        }
    }

    private fun readState(message: String? = null, busy: Boolean = false): OutboxUiState {
        return try {
            val counts = offline.outboxCounts()
            OutboxUiState.Ready(
                entries = offline.outboxEntries(),
                pending = counts.pending,
                inFlight = counts.inFlight,
                done = counts.done,
                failed = counts.failed,
                message = message,
                busy = busy,
            )
        } catch (e: Exception) {
            OutboxUiState.Error(e.message ?: "no se pudo leer la cola")
        }
    }

    /**
     * Attempts every due entry over RCON.
     *
     * Stops early when the server cannot be reached: retrying the remaining
     * entries would only burn their attempt budgets on a link that is down, and
     * the backoff is better spent later.
     */
    fun drain() {
        val current = _state.value as? OutboxUiState.Ready ?: return
        if (current.busy) return
        _state.value = current.copy(busy = true, message = null)

        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) { runDrain() }
            _state.value = readState(message = result)
        }
    }

    private suspend fun runDrain(): String {
        val reclaimed = runCatching { offline.reclaimInterruptedActions() }.getOrDefault(0uL)
        val target = runCatching { core.connectionTarget() }.getOrNull()
            ?: return "No hay destino RCON resoluble."

        val password = runCatching { core.loadConfig().adminPassword }.getOrDefault("")
        val client = RconClient(target.host, target.port.toInt(), password)

        var sent = 0
        var failed = 0
        var stoppedEarly = false

        val due = runCatching { offline.dueActions() }.getOrDefault(emptyList())
        for (action in due) {
            if (action.action != "rcon.command") {
                offline.markActionFailed(action.idempotencyKey, "acción no soportada")
                failed++
                continue
            }
            val command = extractCommand(action.payload)
            if (command == null) {
                offline.markActionFailed(action.idempotencyKey, "payload sin 'command'")
                failed++
                continue
            }

            offline.claimAction(action.idempotencyKey)
            try {
                client.execAsync(command)
                offline.markActionDone(action.idempotencyKey)
                sent++
            } catch (e: Exception) {
                offline.markActionFailed(action.idempotencyKey, e.message ?: "error")
                failed++
                // A transport failure means the link is down; stop rather than
                // spend every entry's budget on an attempt that cannot succeed.
                if (e is java.io.IOException) {
                    stoppedEarly = true
                    break
                }
            }
        }

        val base = "Enviadas $sent, con error $failed. Reclamadas $reclaimed."
        return if (stoppedEarly) "$base Se detuvo: el servidor no responde." else base
    }

    /** Returns a `Failed` entry to the queue so a later drain tries it again. */
    fun retry(idempotencyKey: String) {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                runCatching { offline.retryAction(idempotencyKey) }
            }
            refresh()
        }
    }

    fun pruneDelivered() {
        viewModelScope.launch {
            withContext(Dispatchers.IO) {
                runCatching { offline.pruneDone(86_400u) }
            }
            refresh()
        }
    }

    /**
     * Pulls `command` out of an outbox payload.
     *
     * Hand-parsed rather than pulling in a JSON library: the payload shape is
     * `{"command":"…"}` written by this app and the desktop, and the only value
     * that can contain escapes is the command itself.
     */
    private fun extractCommand(payload: String): String? {
        val key = "\"command\""
        val keyIndex = payload.indexOf(key)
        if (keyIndex < 0) return null
        val colon = payload.indexOf(':', keyIndex + key.length)
        if (colon < 0) return null
        val start = payload.indexOf('"', colon)
        if (start < 0) return null

        val sb = StringBuilder()
        var i = start + 1
        while (i < payload.length) {
            val ch = payload[i]
            when {
                ch == '\\' && i + 1 < payload.length -> {
                    when (payload[i + 1]) {
                        'n' -> sb.append('\n')
                        'r' -> sb.append('\r')
                        't' -> sb.append('\t')
                        '"' -> sb.append('"')
                        '\\' -> sb.append('\\')
                        else -> sb.append(payload[i + 1])
                    }
                    i += 2
                }
                ch == '"' -> return sb.toString()
                else -> {
                    sb.append(ch)
                    i++
                }
            }
        }
        Log.w("OutboxViewModel", "payload had no closing quote: $payload")
        return null
    }
}

package com.arkasa.mobile.presentation

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.arkasa.mobile.data.RconClient
import com.arkasa.mobile.data.RconException
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
 * Console: sends RCON commands and, when the server cannot be reached, **queues
 * them instead of losing them**.
 *
 * That fallback is the whole point of the offline layer. A mobile operator is
 * exactly the person who is away from the machine, on a flaky link, wanting to
 * run `saveworld` or `destroywilddinos` — so a failed send becomes a durable
 * outbox entry with an idempotency key, and the desktop (or a later drain) gets
 * to execute it.
 *
 * The idempotency key is derived from the command text plus the current time
 * bucket, so an accidental double-tap collapses into one queued action while two
 * genuinely separate requests stay distinct.
 */
class ConsoleViewModel(
    private val core: MobileCore,
    private val offline: OfflineCore,
) : ViewModel() {

    private val _state = MutableStateFlow<ConsoleUiState>(ConsoleUiState.Loading)
    val state: StateFlow<ConsoleUiState> = _state.asStateFlow()

    private val lines = mutableListOf<ConsoleLine>()

    init {
        loadTarget()
    }

    private fun loadTarget() {
        _state.value = ConsoleUiState.Loading
        viewModelScope.launch {
            _state.value = withContext(Dispatchers.IO) {
                try {
                    val target = core.connectionTarget()
                    ConsoleUiState.Ready(
                        targetLabel = target.label,
                        targetAddress = "${target.host}:${target.port}",
                        lines = lines.toList(),
                        sending = false,
                        offlineNotice = null,
                    )
                } catch (e: Exception) {
                    ConsoleUiState.Error(e.message ?: "no se pudo resolver el destino RCON")
                }
            }
        }
    }

    /**
     * Sends [command]; on a connection failure, queues it for later.
     *
     * Never throws: a console that crashes on a flaky link would be worse than
     * one that reports the failure and preserves the command.
     */
    fun send(command: String) {
        val trimmed = command.trim()
        if (trimmed.isEmpty()) return
        val current = _state.value as? ConsoleUiState.Ready ?: return
        if (current.sending) return

        _state.value = current.copy(sending = true, offlineNotice = null)

        viewModelScope.launch {
            val target = withContext(Dispatchers.IO) {
                runCatching { core.connectionTarget() }.getOrNull()
            }
            if (target == null) {
                finishSend(ConsoleLine(trimmed, "no se pudo resolver el destino", ConsoleLine.Kind.Failed))
                return@launch
            }

            val client = RconClient(target.host, target.port.toInt(), adminPassword())
            val outcome = runCatching { client.execAsync(trimmed) }

            outcome.fold(
                onSuccess = { body ->
                    finishSend(ConsoleLine(trimmed, body.ifBlank { "(sin respuesta)" }, ConsoleLine.Kind.Sent))
                },
                onFailure = { error ->
                    // A refused password is a configuration problem the operator
                    // can fix; a socket failure is the offline case worth queueing.
                    val queued = when (error) {
                        is RconException -> queueCommand(trimmed) && error.message?.contains("rechazada") != true
                        is java.net.SocketTimeoutException, is java.io.IOException -> queueCommand(trimmed)
                        else -> false
                    }
                    if (queued) {
                        finishSend(
                            ConsoleLine(trimmed, "sin conexión · encolado para reintentar", ConsoleLine.Kind.Queued),
                            offlineNotice = "El servidor no responde; el comando quedó en la cola.",
                        )
                    } else {
                        finishSend(ConsoleLine(trimmed, error.message ?: "error", ConsoleLine.Kind.Failed))
                    }
                },
            )
        }
    }

    fun clearTranscript() {
        val current = _state.value as? ConsoleUiState.Ready ?: return
        lines.clear()
        _state.value = current.copy(lines = emptyList(), offlineNotice = null)
    }

    private fun finishSend(line: ConsoleLine, offlineNotice: String? = null) {
        lines += line
        val current = _state.value as? ConsoleUiState.Ready ?: return
        _state.value = current.copy(
            lines = lines.toList(),
            sending = false,
            offlineNotice = offlineNotice,
        )
    }

    /** Queues the command durably; returns whether it is now in the queue. */
    private suspend fun queueCommand(command: String): Boolean = withContext(Dispatchers.IO) {
        runCatching {
            val key = "rcon:${command}:${System.currentTimeMillis() / 2_000}"
            val payload = """{"command":${escapeJson(command)}}"""
            offline.enqueueAction(key, "rcon.command", payload)
            true
        }.getOrDefault(false)
    }

    private fun adminPassword(): String =
        runCatching { core.loadConfig().adminPassword }.getOrDefault("")

    private fun escapeJson(value: String): String {
        val sb = StringBuilder("\"")
        for (ch in value) {
            when (ch) {
                '"' -> sb.append("\\\"")
                '\\' -> sb.append("\\\\")
                '\n' -> sb.append("\\n")
                '\r' -> sb.append("\\r")
                '\t' -> sb.append("\\t")
                else -> if (ch < ' ') sb.append("\\u%04x".format(ch.code)) else sb.append(ch)
            }
        }
        return sb.append("\"").toString()
    }
}

# Architecture Audit — Sesión 6 / P6

> Honesto **gapanálisis** entre el patrón ARK ASA v2.1 que tenemos hoy
> y lo que la comunidad open-source (OpenClaw / Hermes Agent / Agent
> Harness Core / Mastra / OpenSAGE / Bullwork) considera "best practice"
> en arquitectura de plugins multi-channel + agent runtime seguro.

## 1. Lo que tenemos hoy (verificado contra main)

| Capacidad | Estado | Dónde |
|---|---|---|
| **Plugin registry dinámico** | ✅ Implementado en P1 (commit 3ba089b) | `src-tauri/src/plugins/{mod.rs, registry.rs, pluginhub.rs}` |
| **Add/remove de plugins sin recompilar** | ✅ Solo Convex/Vercel cargados; enable_plugin/disable_plugin via TOML | `~/.ark-asa/plugins/registry.toml` |
| **VPS connection providers como plugins** | ✅ Catalog de 7 providers (Oracle/Hetzner/DO/Selfhost/AWS/Azure/GCP) | `src-tauri/src/plugins/connection.rs` |
| **AI model plugins** (8 OpenAI-compatible) | ✅ OpenAI/Cerebras/NIM/llama.cpp/Ollama/vLLM/LM Studio/Custom | `src-tauri/src/plugins/model.rs` |
| **UI Plugin Hub tab** | ✅ Frontend tab wired en `OptionsModal` | `frontend/src/components/options/PluginsTab.tsx` |
| **Identity 7-axis** | ✅ Implementado en Sesión 2b | `integrations/identity.rs` |
| **Receipts ledger JSONL con fsync** | ✅ Implementado | `receipts/mod.rs` |
| **Self-host (Pi/NUC/WSL2/macOS)** | ✅ 7 hardware classes + 3 manuales | `docs/HOSTING_SELFHOSTED.md` |
| **Network/Tailscale wizard** | ✅ CGNAT heuristic + setup form | `docs/NETWORK_TAILSCALE.md` |
| **Convex/Vercel one-click** | ✅ Implementado | `docs/{CONEX,VERCEL}.md` |
| **0 warnings** | ✅ logrados en P5 (commit 965f4d9) | — |

Tests: **97/97 passing** en cargo test --lib. Frontend tsc --Emit limpio.

---

## 2. Lo que OpenClaw / Hermes Agent / Agent Harness Core hacen bien (que NO tenemos)

| Patrón | Cuál es la referencia | Brecha en nuestro código | Severidad |
|---|---|---|---|
| **Manifest en runtime (no compilación)** | OpenClaw: cada plugin es un crate + manifest JSON separado; runtime lo descubre. | Nuestro `register_default_plugins` enumera Convex/Vercel en código (hardwired). Habilitamos add/remove pero la carga inicial es estática. | **Medio** — limita extensibilidad futura |
| **Plugin con procesos aislados** | OpenClaw: cada plugin corre en sidecar WASM/MicroVM con permisos capados por capability. | Telegram/Discord/Slack corren `tokio::spawn` con todo el proceso del desktop. Si un plugin tiene bug de castigo, cae el proceso entero. | **Medio** — riesgo de disponibilidad |
| **Capability-based permissions declarativas** | Agent Harness Core: cada plugin declara `<capability name="send_message"/>` y el runtime valida antes de ejecutar. | Nuestro `PluginCapability::MessagesRecv/MessagesSend/RequiresOAuth/RequiresSecrets` enum existe, pero no se enforce — sólo es display metadata. | **Medio-alto** — superficie de ataque latente |
| **Plugin Sandbox (no acceso directo al host FS / network)** | Hermes: cada plugin accede via `Provider<R>` recursos tipados. | Nuestro `secret_store::read` abre disco; cada plugin escribe a `registry.toml` directamente. | **Bajo-Medio** — internal plugins son trusted (== mismo Origin), pero si cargamos plugins de terceros el modelo rompe. |
| **OTA Update con signature verification** | OpenSAGE, Bullwork: actualizan el manifest firmado con ed25519/rsa. | NO tenemos update chaining. Bots / vercel / convex son versionados en source. | **Bajo** — desktop app updates via Tauri auto-updater fuera de este repo. |
| **OpenTelemetry trace propagation cross-plugin** | Mastra: un `tracer` se pasa por header, cada plugin es un span. | Nuestro Receipts JSONL ledger funciona como audit log per-event pero **no es tracing**. Si hay un trace_id cross-hop, se trunca. | **Medio** — debugger en multi-bug scenarios |
| **eBPF / sandboxing for skills** | OpenClaw consumes eBPF hooks (via Falco/Tetragon) para limitar syscalls del plugin. | No aplicable a Windows + WSL2 mix. Sin sandboxing real. | **Bajo** (no usaremos) |
| **Versioning + semver constraint del plugin** | Hermes: cada plugin declara `[1.0.0, 2.0.0)`. | Convex/Vercel tienen `<&'static str>` ids sin version. Registries futuras podrían romper. | **Bajo** —semver vendría con V2.1+ |

---

## 3. Lo que otros hacen mal (que evitamos hacer)

| Anti-pattern | Quién lo hace | Cómo lo evitamos |
|---|---|---|
| **Reempacar un framework-Chat en cada plugin** | Muchos market-place chat-adapter: cada uno inventa su workflow de polling/retry/backoff/SLAs. | Nosotros tenemos UN command_router (Sesión 2b) y UN receipt_emit (Sesión 2b). Cada plugin adapter es delgado. |
| **OAuth custom server / reimplementar Auth0** | Muchos proyectos (slack-telegram-bridge, etc.) reescriben spec flows adentro del plugin. | PluginHub registra TODOS los plugins como **CLI-bridge** (delegamos a la CLI oficial del servicio). |
| **Pegar el secret en stdout/logs** | Plugins mal escritos a veces log el bot_token. | Nuestro secret_store escribe `0600`/`NTFS-ACL`. `paste_convex_deploy_key` limpia después de tests. **Sí ocurre en `npx convex login` stdout — un riesgo en logs/debug, parcialmente mitigado por el flujo `paste_*_key` inyectable.** |
| **Worker busy-loop o polling 1Hz** | Plugins con busy-loop `loop { sleep(1); check }`. | Nuestro `Discord` y `Telegram` usan `WebSocket` / long-poll con coalescing `last_cmd_at: Instant` rate-limit. No tenemos busy-loop. |
| **Estados globales mutables sin locking** | Plugins típico: `Arc<Mutex<HashMap>>` global. | TelegramBot guarda `parking_lot::Mutex` por `chat_id` para rate-limit; `integration::command_router` pasa `RCO dispatch` por `Arc<dyn Fn>`. |

---

## 4. Brecha numérica por pilar

Reviso cada pilar clásico y respondo con honestidad:

| Pilar | OpenClaw benchmark | Nuestro estado | Gap |
|---|---|---|---|
| **Plugin discovery** | Manifest en disco | Toml + Vec<PluginEntry> estático | +1 día |
| **Plugin isolation** | Process-level WASM sandbox | Same-process tokio task | +1-2 semanas con Taurus / wasmtime |
| **Audit / receipts** | OpenTelemetry + audit como sidecar | JSONL ledger daily-rotated, fsync cada write | Marginal |
| **Concurrent multi-bot dispatch** | 1 command_router universal | Idem (command_router.rs) | Cero |
| **Allow-list / RBAC** | Group → plugin ACL con deny rules | `ChannelBinding::resolve()` con admins_only allowlist | Aceptable; pat-down con deny not exhaustive |
| **Lifecycle** | OTA firmado, canary deploy | Tauri auto-updater (externo al repo) | Aceptable |
| **Crash resilience** | Plugin se reinicia automático | Nuestro `error in plugin` no hace nada; tokio handle cae, no avisa | +1 día |
| **Telemetry** | OTLP + service name + version | JSONL sin OpenTelemetry export | +2 semanas |
| **Distribution** | Rust crate registry indexado por id / version | Compilación artesanal por main | OK para uso interno |

---

## 5. Riesgos identificados (clasificación por gravedad)

### Críticos hoy
- **Ninguno identificado**.

### Altos (deuda futura, no blockers v2.1)
1. **Cero enforcement de `PluginCapability`.** El enum es metadata-only. Si en el futuro alguien carga un plugin externo (no built-in), el runtime NO le negará `send_message` aunque diga `MessagesRecv-only`. Plan: añadir `gateway.rs` que envuelva el dispatch y valide `capabilities.matches(cmd.kind)`.
2. **Slack events-bot-id check está OK, pero `msg.text` sin sanitizar** se mete al run_with_receipts → parser. Si el mensaje es JSON inválido, el parser se ahoga. Plan: envolver en `match` en `parser.rs::analyze_text`.
3. **TelegramBot rate-limit** está por chat_id pero no por user (un usuario puede saltarse el límite cambiando de chat). Plan: añadir `messages_per_user` con TTL window.

### Medios
4. **Network/Tailscale CGNAT heuristic** puede dar falsos positivos (IP pública rota por minutos o captive portals). Plan: marcar como "advisory" en UI, dejar al operador decidir.
5. **PluginHub enable/disable no checks si el plugin estaba vivo**. Si Convex está ejecutando un `JoinHandle`, disable no lo aborta. Plan: `abort_handle` cargado en `PluginEntry` para future-proof.
6. **Self-host WSL2 portproxy** depende de Windows feature `netsh interface portproxy`; cambios de firewall del operador (McAfee, Kaspersky) podrían romperlo. Plan: documentar en `docs/COMMON_ISSUES.md`.

### Bajos
7. **AI `AiClient::ask`** usa `reqwest` con `system-prompt` literal. No hay `system_prompt_v2` para delimitar inyecciones. Plan: prompt-eng-template + escape de triple-backticks.
8. **`--advertise-tags` en Tailscale** está hardcoded como `TAG:` + label. Si el operador quiere `tag:arkasa-prod`, lo soportamos pero hay que documentarlo en lugar.

---

## 6. Recomendaciones — roadmap post-v2.1.0

En orden de impacto/coste:

| Prioridad | Tarea | Coste | Impacto |
|---|---|---|---|
| P1 | `PluginGateway` con capability enforcement | 1 d | Cierra el gap "no enforcement" (riesgo alto) |
| P2 | Pulse telemetry: `telemetry::Emitter` con OTLP/gelf | 4 d | Visibilidad operativa |
| P3 | `provider_schema.toml` para self-host (qué archivos necesitan intervención manual) | 1 d | Reduce onboarding friction |
| P4 | `slack_id_nick` cache y `messages_per_user` rate-limit | 0.5 d | Anti-stress |
| P5 | WASM plugin runtime (Tauri + wasmtime) para 3rd-party plugins | 2-4 semanas | Habilita verdadera extensibilidad |
| P6 | Build Tauri -> Apple Silicon notarization pipeline (CI) | 2 d | Acelera macOS release |

---

## 7. Por qué este documento

Cuando te planteé el compromiso de honestidad al inicio de la sesión:

> "los 4 blocks v2.1.0 deben cerrarse antes de cortar el tag"

Y elegiste seguir con P5/P1/P2/P3/P4 sin cortar el tag, este archivo **es la prueba contractual** de que:

1. Reconocemos dónde estamos ya maduros.
2. Reconocemos dónde estamos por debajo de Hermes, OpenClaw, et al.
3. Catalogamos deuda futura con severidad y coste estimado.

El release `v2.1.0` puede acontecer cuando **P1..P5** (Sesiones 2-6) estén shipped, **independientemente** del gap P6 de abajo. La auditoria queda como guía para `v2.1.1` o `v2.2`.

---

## 8. Conclusión honesta

El producto (Sesiones 2-6, branches anteriores) entrega **valor real**:
- Multi-bot admin con audit completo.
- Self-host rápido en Pi / NUC / WSL2.
- Backend & frontend deploy one-click.
- Plugin Hub inicial con toggle runtime.
- 0 warnings, **97 tests**, arquitectura clara.

Lo que **falta** (nivel Hermes-grade) es sandboxing, OTLP, y un verdadero marketplace de plugins externos fuera del repo. Eso es **v2.2 / v3** territorio.

**Conclusión**: v2.1.0 puede cerrarse como RC. Sin urgencia de "feature parity con OpenClaw". Cuando el usuario-multi-cliente (no el operador single-Mac) aparezca en 2026 Q4, se reabre P6.

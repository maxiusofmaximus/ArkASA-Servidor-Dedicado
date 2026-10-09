# Prometheus metrics & Grafana dashboard

The app exposes a Prometheus/OpenMetrics endpoint on the same loopback HTTP API
it already runs (`127.0.0.1:8765`), so an operator can monitor the ARK server
with the standard stack.

## Endpoint

```
GET http://127.0.0.1:8765/api/v1/metrics
Authorization: Bearer <admin token>
```

Bearer-authenticated like every other endpoint (`/api/v1/health` is the only
public one). Get the token from **Options → Remote Admin** in the desktop app.

The response is `text/plain; version=0.0.4; charset=utf-8` (Prometheus
exposition format 0.0.4).

### Metrics

| Metric | Type | Meaning |
|---|---|---|
| `arkasa_info{version}` | gauge | always `1`; carries the app version |
| `arkasa_server_running{session}` | gauge | `1` if `ArkAscendedServer.exe` is running |
| `arkasa_process_memory_mb{session}` | gauge | resident memory of the server process |
| `arkasa_server_fps{session}` | gauge | server FPS scraped from `ShooterGame.log` |
| `arkasa_udp_conns{session}` | gauge | public UDP connections held by the process (Windows) |
| `arkasa_maps_total{session}` | gauge | configured cluster maps |
| `arkasa_mods_active{session}` | gauge | active mods |
| `arkasa_max_players{session}` | gauge | configured max players |
| `arkasa_game_port{ *_port }` | gauge | game / query / RCON ports |

**Honest absence:** a metric whose value is not available right now (FPS before
the log has seen a `Stat FPS` line, memory if the live parse failed, UDP conns
on Linux) is **not emitted at all** — the API never paints a phantom zero.

## Prometheus scrape config

`prometheus.yml`:

```yaml
scrape_configs:
  - job_name: arkasa
    scrape_interval: 15s
    authorization:
      credentials: "<admin token>"
    static_configs:
      - targets: ["127.0.0.1:8765"]
    metrics_path: /api/v1/metrics
```

The endpoint is bound to loopback only. To scrape from another host, put the
app behind a reverse proxy with TLS (never expose `8765` directly — see the
security notes in `RESEARCH_NEW_IDEAS.md`).

## Grafana dashboard

Import [`grafana/ark-asa-dashboard.json`](./grafana/ark-asa-dashboard.json)
(Dashboards → New → Import → upload JSON, pick your Prometheus datasource). It
ships panels for server state, memory, FPS, active mods, UDP connections and a
config/ports table.

## CLI parity

`ark-config` does not talk to Prometheus, but the same data is available
locally:

```bash
ark-config status        # running state + per-map
ark-config config        # ports, session, etc.
```

A `--json` output mode is tracked in `RESEARCH_NEW_IDEAS.md` (idea #14,
"CLI parity + machine-readable output").

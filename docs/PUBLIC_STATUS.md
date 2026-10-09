# Public status page & embed badges

A read-only, **sanitised** status surface you can publish for your community
(website, Discord, forum signature) without exposing the server-management API.

## It is opt-in

Nothing is exposed by default. The endpoints return `404` until you set:

```
ARK_ASA_PUBLIC_STATUS=1
```

for the app process (values `1`, `true`, `yes`, `on` are accepted). When set,
the loopback auth middleware exempts **only** the `/api/v1/public/*` paths;
everything else still requires the bearer token.

## Endpoints

### `GET /api/v1/public/status`

```json
{
  "app": "ark-asa-config",
  "app_version": "2.1.0",
  "session_name": "ServidorMax",
  "status": "RUNNING",          // or "OFFLINE" / "UNKNOWN"
  "running": true,
  "maps_total": 3,
  "mods_active": 15,
  "max_players": 70,
  "platform": "ALL",
  "fps": 30,
  "updated_ms": 1790580798027
}
```

No authentication. **Never** contains paths, ports, passwords or tokens.

### `GET /api/v1/public/badge.svg`

A shields.io-style SVG badge, optional `?label=`:

```html
<img src="http://your-host/api/v1/public/badge.svg?label=ServidorMax" alt="ARK status">
```

```markdown
![ARK status](http://your-host/api/v1/public/badge.svg?label=ServidorMax)
```

Green **RUNNING** / red **OFFLINE**, no external font or image dependency.

## Exposing it safely

The loopback API binds `127.0.0.1` only. To publish it:

1. Put it behind a reverse proxy (nginx/Caddy/Traefik) with **TLS**.
2. Proxy **only** `/api/v1/public/` — never the rest of `/api/v1/`.
3. Rate-limit and cache the badge (it is cheap, but bots hammer embeds).

Do **not** expose port `8765` directly to the internet (see the security notes
in `RESEARCH_NEW_IDEAS.md`).

## Example: nginx

```nginx
location /ark/status/ {
    proxy_pass http://127.0.0.1:8765/api/v1/public/;
    proxy_set_header Host $host;
    add_header Cache-Control "public, max-age=15";
}
```

Then `https://example.com/ark/status/status` and
`https://example.com/ark/status/badge.svg?label=ServidorMax`.

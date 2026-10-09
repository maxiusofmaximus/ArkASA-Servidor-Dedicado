# Custom validators ("validator plugins")

The six built-in validators (ports, passwords, player count, mods, paths,
multipliers) are compiled in. On top of them, the app evaluates **operator-defined
declarative rules** loaded at runtime from:

```
%APPDATA%\ARK ASA Config Manager\custom_validators.toml
```

No recompilation, no code execution, no scripting sandbox — just data. Rules run
in the desktop UI's validation, in `validate_config`, and in the CLI
(`ark-config validate`). A rule that doesn't apply is reported as an error, so
typos fail loudly instead of silently passing.

## Rule format

Each rule targets a config field by its **dot path** and applies one check.

```toml
[[rules]]
id      = "min-players"                       # unique id (shown in the error list)
field   = "gameplay.max_players"              # TOML path into the config
kind    = "range"
min     = 5
max     = 200
message = "El servidor debe admitir entre 5 y 200 jugadores"   # optional
```

The `message` is optional; when omitted a sensible default is generated.

## Supported checks (`kind`)

| `kind` | Parameters | Meaning |
|---|---|---|
| `not_empty` | — | string/array/null must not be empty |
| `min_length` | `min_len` | string or array length ≥ `min_len` |
| `max_length` | `max_len` | string or array length ≤ `max_len` |
| `range` | `min`, `max` (numbers, either optional) | numeric field within bounds |
| `one_of` | `allowed = [...]` | string must be one of `allowed` |
| `contains` | `value` | string must contain `value` |
| `starts_with` | `value` | string must start with `value` |
| `ends_with` | `value` | string must end with `value` |
| `numeric` | — | string of ASCII digits only |
| `must_be_true` | — | boolean must be `true` |
| `must_be_false` | — | boolean must be `false` |

`field` walks objects and arrays; array elements are addressed by index, e.g.
`mods.active_mods.0`.

## Examples

```toml
# Reject the default/weak admin password outright
[[rules]]
id    = "strong-admin-pw"
field = "identification.admin_password"
kind  = "min_length"
min_len = 12
message = "La contraseña de admin debe tener al menos 12 caracteres"

# Session name must not be blank
[[rules]]
id    = "named-server"
field = "identification.session_name"
kind  = "not_empty"

# Hard cap on cluster size
[[rules]]
id    = "cluster-cap"
field = "cluster_maps"
kind  = "max_length"
max_len = 5
message = "No se permiten más de 5 mapas en el cluster"

# First mod id must be numeric (sanity check on the mods list)
[[rules]]
id    = "first-mod-numeric"
field = "mods.active_mods.0"
kind  = "numeric"
```

## Managing rules

- **CLI:** edit the file directly, then run `ark-config validate`.
- **Tauri commands:** `list_custom_validators`, `upsert_custom_validator`,
  `remove_custom_validator`, `custom_validators_path`.

## Notes

- A missing or malformed `custom_validators.toml` is treated as "no custom
  rules" — it never bricks the app.
- `upsert_custom_validator` replaces a rule with the same `id`.
- Rules are per-machine (they live in the config dir), not per-profile. If you
  need per-profile rules, target profile-specific fields or run the CLI with a
  separate `--config`.

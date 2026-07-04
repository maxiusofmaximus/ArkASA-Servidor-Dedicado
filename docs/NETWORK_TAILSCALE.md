# Network & Tailscale Wizard (alpha toward v2.1)

> The desktop app now detects CGNAT, surfaces the operator's public
> IPv4 + Tailscale IP, and runs `tailscale up` from a pasted
> auth-key. This closes **blocker #4 of v2.1.0** — the wizard.

## What "CGNAT suspect" means

Carrier-Grade NAT (CGNAT) is when your ISP puts you behind an
extra layer of NAT — your "public" IPv4 in *your* router UI isn't
actually the IPv4 internet sees. The desktop uses a heuristic:

| State | Interpretation |
|---|---|
| Public IPv4 reachable (`api4.ipify.org`) | Port forwarding probably works |
| Public IPv4 unreachable | CGNAT or no internet |
| Tailscale installed & IP `100.64-127.x.y` reachable | Operator can use the Tailscale IP as a connection entry |
| Tailscale installed but no IP | Operator has it but never ran `tailscale up` |

The wizard surfaces all four signals in the General → Public network
& Tailscale section.

## What you'll do as the operator

1. **Open the desktop app → Options → General → Internet.**
2. **A new section "Public network & Tailscale" appears just below.**
   It shows:
   - Public IPv4 (green if reachable; amber if not)
   - Tailscale CLI installed (green / amber)
   - Tailscale IP if any
   - CGNAT heuristic (green if no CGNAT; amber if suspect)
3. **If CGNAT is suspected**, the wizard offers:
   - **INSTALL TAILSCALE** if not installed (deep link to the
     download URL for your platform).
   - **Paste Auth Key + Tailscale hostname** form (only shown when
     Tailscale is installed but not yet up).
4. **Click SET UP TAILSCALE**. The desktop app spawns
   `tailscale up --authkey <key> --hostname <host>` and re-polls
   `tailscale ip -4`. When it returns, the wizard shows the new
   `100.x.x.x` IP and a success message:
   > 🟢 Tailscale is up. Share this IP with your friends: `100.100.100.50`

5. **Friend flow.** Your friends install Tailscale, you approve
   them on the Tailscale admin panel, and they connect to
   `<100.x.x.x>:7777` (UDP) just like a normal Steam IP.

## What the app does internally

```
tailscale_status_combined()              # Tauri command (called on mount)
    │
    ├─ detect_public_ip()                 # api4.ipify.org probe
    ├─ detect_tailscale_ip()              # 'tailscale ip -4' OR parse ipconfig/ip cmd
    ├─ detect_tailscale_cli()             # which tailscale / where tailscale / path probe
    └─ cgnat_suspect(public, tailscale)   # pure heuristic, no IO

tailscale_setup(auth_key, hostname, dns_label)  # Tauri command (one-shot)
    │
    ├─ reject empty inputs (UI catches the same)
    └─ spawn: tailscale up --authkey <key> --hostname <host>
                [--advertise-tags tag:<dns_label>]
       captures stdout/stderr
       re-polls 'tailscale ip -4' and surfaces the result.
```

All of this is fully synchronous from the operator's perspective;
no OAuth dance, no callbacks, no Tailscale account creation in the
app. Just paste a key you've already minted.

## Auth keys: how to mint one

1. **Sign in** at <https://login.tailscale.com>.
2. Go to **Settings → Keys → Generate auth key**.
3. Recommended settings for this app:
   - **Reusable:** ON (so we can refresh after a re-install)
   - **Ephemeral:** OFF (we want the node to persist)
   - **Tags:** `tag:arkasa` if you've set up an ACL tag in your
     tailnet policy. Otherwise leave empty.
4. **Copy the key** (`tskey-auth-…`) and paste it into the desktop
   app's Tailscale wizard.

## Why we don't OAuth

Tailscale does have OAuth client flows, but they're scoped for
*servers running a tailnet of their own*. The desktop app is
just an operator running under the operator's own tailnet. OAuth
would force the operator to create a brand-new tailnet in their
name. Auth keys are simpler and what Tailscale recommends for
self-hosted apps today.

## What about public IPv4 + no Tailscale?

If CGNAT is **not** detected (the `Public IPv4` row is green), the
wizard shows a green badge "● connected" but doesn't push any
deployment. Port forwarding on the router is the canonical path —
see `docs/NETWORK_SETUP.md § 7` for the rest. The Tailscale
detection still runs in the background so the section still
responds if the operator changes ISP later.

## What about iPhone / Android?

ARK players running the Web Admin on their phone should:
1. Install Tailscale from the App Store / Play Store.
2. Sign in to the same tailnet.
3. Connect to `<100.x.x.x>:7777` (UDP) from the Web Admin.

The Tailscale app routes all phone traffic into the tailnet
transparently, so the player doesn't see a different IP — UDP
just works.

## Common pitfalls

- **Auth key rejected.** The wizard surfaces the stderr verbatim
  to the panel. Common cause: your tailnet is *impaired* by an
  ACL policy that doesn't allow new nodes tagged `tag:arkasa`. Add
  the ACL entry or leave Tags empty when minting the key.
- **`tailscale up` times out.** Tailscale's hipcheck server can be
  unreachable on captive-portal Wi-Fi. Switch to a different
  network first.
- **Operator wants a MagicDNS hostname (e.g. `arkasa-pi5.tail...`).**
  Tailscale generates one automatically once `--hostname` is set.
  Look up the current value in the admin panel.

## Where this lands in CHANGELOG and TODO

- **CHANGELOG § "Unreleased → Public network & Tailscale wizard
  (alpha toward v2.1)"** flags blocker #4 of v2.1.0 as shipped.
  Once that's done, **all four v2.1.0 blockers are closed**:
  1. ✅ Convex deploy flow
  2. ✅ Vercel deploy flow
  3. ✅ VPS self-host (Pi / NUC / WSL2)
  4. ✅ Tailscale wizard
- **TODO.md** marks the Open work as resolved. Reviewer can now
  choose to cut `v2.1.0` (first non-alpha tag since `v2.1.0-alpha.2`).

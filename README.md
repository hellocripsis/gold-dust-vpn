# Gold Dust VPN (control plane prototype)

Gold Dust VPN is a small Rust command-line control plane that simulates an **Oxen-first, Tor-fallback** routing policy.

It does **not** move packets. Instead, it answers the question:

> “Given the current health of my Oxen nodes and Tor exits, which backend *would* I send this connection to?”

This makes it a clean portfolio project for backend / systems work:
- config file
- CLI with subcommands
- simple routing policy
- structured status output

---

## Features (v0.1)

- **Backends**:
  - Two simulated Oxen nodes
  - One simulated Tor exit
- **Health snapshot**:
  - Latency (ms)
  - Failure rate (0.0 – 1.0)
  - Enabled flag from config
- **Routing policy**:
  - Prefer enabled Oxen nodes with the lowest latency
  - If all Oxen are disabled, **fall back to Tor**
  - If everything is disabled, return an error

---

## Config

Gold Dust reads a simple TOML config from `gold-dust-vpn.toml` in the project root:

```toml
[backends]
oxen_enabled = true
tor_enabled = true

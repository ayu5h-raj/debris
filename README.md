# Debris

Minimal Mac storage cleaner. Finds and removes leftover files from uninstalled apps, orphaned containers, dead launch agents, and dev tool caches.

![macOS](https://img.shields.io/badge/macOS-12%2B-blue) ![License](https://img.shields.io/badge/license-MIT-green)

## What it scans

| Category | What's detected |
|---|---|
| **Orphaned App Data** | `~/Library/Application Support` dirs with bundle IDs of uninstalled apps (known DB + heuristic) |
| **Containers** | `~/Library/Containers` and `~/Library/Group Containers` entries with no matching installed app |
| **Launch Agents** | `~/Library/LaunchAgents` plists whose binary no longer exists |
| **Dev Caches** | npm, yarn, pip, uv, Cargo, Go modules, Puppeteer, node-gyp, Docker |

## Install

```bash
brew tap ayu5h-raj/tap
brew install debris
```

Then launch:

```bash
debris
```

## Build from source

Requires Rust 1.78+.

```bash
git clone https://github.com/ayu5h-raj/debris
cd debris
cargo run -p debris-app
```

## How it works

`debris-core` runs four parallel scanners on startup:

1. **Known DB** (`debris-db`) — matches `~/Library/Application Support` against a curated list of app bundle ID → file path mappings
2. **Heuristic** — flags any `Application Support` subdir whose name looks like a bundle ID (`com.x.y`) with no matching `.app` in `/Applications`
3. **Containers** — reads `MCMMetadataIdentifier` from each container's metadata plist and cross-references against installed apps
4. **Launch Agents** — checks `Program`/`ProgramArguments` in each plist; flags it if the binary is missing

Results stream to the UI via an `mpsc` channel as they arrive.

## Project structure

```
debris-db/      curated app → paths database (TOML, compiled in)
debris-core/    scanners, cleaner, disk info
debris-app/     egui/eframe GUI
Formula/        Homebrew formula
```

## License

MIT

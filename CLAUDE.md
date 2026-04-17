# Debris — Claude Code context

## Commands

```bash
cargo build -p debris-app          # build GUI app
cargo run -p debris-app            # run GUI app
cargo test                         # all tests (25 core + 3 db)
cargo test -p debris-core          # core tests only
cargo clippy -- -D warnings        # lint (zero warnings required)
open target/debug/debris           # open already-built binary
cargo run -p debris-tui            # run TUI app
cargo test -p debris-tui           # TUI tests only
```

## Architecture

Three crates:

- **`debris-db`** — compile-time TOML database mapping known app bundle IDs to their leftover paths. `load_app_entries()` returns `Vec<AppEntry>`.
- **`debris-core`** — all scanning logic, cleaner, disk info. Exports via `lib.rs`. Key types: `OrphanItem`, `DevCacheItem`, `LaunchAgentItem`, `ScanEvent`.
- **`debris-app`** — egui 0.34 / eframe 0.34 GUI. `SweepApp` in `app.rs` holds all state; `logic()` drains the scan channel, `ui()` renders.
- **`debris-tui`** — ratatui 0.29 / crossterm 0.28 TUI. `TuiApp` in `app.rs` holds all state; `event.rs` maps keys to mutations; `ui/` renders each panel.

## Scanner pipeline

`run_scan(home, applications)` spawns a thread that runs scanners sequentially and sends events over `mpsc`:

1. `scan_known` — known DB matches
2. `scan_heuristic` — bundle-ID-shaped dirs in Application Support not in installed apps
3. `scan_containers` — sandboxed containers + group containers
4. `scan_dev_caches` — hardcoded list of dev tool cache paths
5. `scan_launch_agents` — LaunchAgents plists with missing binaries

`installed_bundle_ids()` is shared via `orphan/util.rs` — used by both heuristic and containers scanners.

## Key conventions

- All size values are raw bytes (`u64`); `format_bytes()` in each crate's `ui/mod.rs` formats for display
- List panels in debris-app sort by size descending at render time (snapshot sort, not in-place)
- Deletions are deferred out of scroll loops to avoid borrow checker conflicts
- Confirmation state lives on `SweepApp`: `confirm_delete` (orphans bulk), `confirm_clear_cache: Option<usize>` (dev caches), `confirm_delete_agent: Option<usize>` (launch agents)
- No clippy warnings allowed (`-D warnings` in CI)

## Adding a new dev cache

Add one line to `CACHES` in `debris-core/src/dev_cache.rs`:

```rust
CacheDef { name: "My Tool", relative_path: ".cache/my-tool" },
```

## Adding a new scanner

1. Create `debris-core/src/my_scanner.rs` with item type + `scan_*` fn
2. Export from `lib.rs`
3. Add `MyScannerFound(MyItem)` variant to `ScanEvent` in `scanner.rs`
4. Wire into `run_scan` thread
5. Add field to `SweepApp`, drain in `logic()`, create `ui/my_panel.rs`, add sidebar nav

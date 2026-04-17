# Debris v1.1 — Containers + Launch Agents Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two new detection categories to debris: `~/Library/Containers` + `~/Library/Group Containers` orphan scanning, and orphaned LaunchAgent plist detection.

**Architecture:** Two additive modules in `debris-core` — `orphan/containers.rs` (reuses existing `OrphanItem` type, adds `OrphanSource::Containers` variant) and `launch_agent.rs` (new `LaunchAgentItem` type and panel). A shared `orphan/util.rs` extracts the `installed_bundle_ids()` helper currently private in `heuristic.rs` so both scanners can use it. The UI adds one new sidebar section (Launch Agents) and extends the overview breakdown.

**Tech Stack:** Rust, plist crate (already a dep), egui 0.34 / eframe 0.34, same patterns as existing scanners.

---

## File Map

```
debris-core/src/orphan/
├── mod.rs               MODIFY — add OrphanSource::Containers, pub mod containers, pub mod util
├── util.rs              CREATE — shared installed_bundle_ids() helper
├── heuristic.rs         MODIFY — use util::installed_bundle_ids instead of private fn
└── containers.rs        CREATE — scan_containers()

debris-core/src/
├── launch_agent.rs      CREATE — LaunchAgentItem, scan_launch_agents()
├── scanner.rs           MODIFY — add containers + launch agents to run_scan, new ScanEvent variant
└── lib.rs               MODIFY — export LaunchAgentItem, scan_launch_agents

debris-app/src/
├── app.rs               MODIFY — add launch_agents field, Section::LaunchAgents
└── ui/
    ├── mod.rs           MODIFY — add pub mod launch_agents
    ├── sidebar.rs       MODIFY — add Launch Agents nav item
    ├── overview.rs      MODIFY — add launch agents bar
    └── launch_agents.rs CREATE — draw_launch_agents() panel
```

---

### Task 1: Extract shared `installed_bundle_ids` utility

**Files:**
- Create: `debris-core/src/orphan/util.rs`
- Modify: `debris-core/src/orphan/heuristic.rs`
- Modify: `debris-core/src/orphan/mod.rs`

- [ ] **Step 1: Write failing test for util**

Create `debris-core/src/orphan/util.rs` with tests only:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_empty_applications_returns_empty_set() {
        let apps = tempdir().unwrap();
        let ids = installed_bundle_ids(apps.path());
        assert!(ids.is_empty());
    }

    #[test]
    fn test_app_without_plist_skipped() {
        let apps = tempdir().unwrap();
        // .app dir with no Contents/Info.plist
        fs::create_dir_all(apps.path().join("Foo.app/Contents")).unwrap();
        let ids = installed_bundle_ids(apps.path());
        assert!(ids.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/ayushraj/Documents/github/debris
cargo test -p debris-core orphan::util
```
Expected: FAIL — `installed_bundle_ids` not defined.

- [ ] **Step 3: Implement util.rs**

`debris-core/src/orphan/util.rs`:
```rust
use std::{collections::HashSet, fs, path::Path};

pub fn installed_bundle_ids(applications: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();
    let Ok(entries) = fs::read_dir(applications) else { return ids };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "app") {
            let plist_path = path.join("Contents/Info.plist");
            if let Ok(val) = plist::from_file::<_, plist::Value>(&plist_path) {
                if let Some(id) = val
                    .as_dictionary()
                    .and_then(|d| d.get("CFBundleIdentifier"))
                    .and_then(|v| v.as_string())
                {
                    ids.insert(id.to_lowercase());
                }
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_empty_applications_returns_empty_set() {
        let apps = tempdir().unwrap();
        let ids = installed_bundle_ids(apps.path());
        assert!(ids.is_empty());
    }

    #[test]
    fn test_app_without_plist_skipped() {
        let apps = tempdir().unwrap();
        fs::create_dir_all(apps.path().join("Foo.app/Contents")).unwrap();
        let ids = installed_bundle_ids(apps.path());
        assert!(ids.is_empty());
    }
}
```

- [ ] **Step 4: Update heuristic.rs to use util**

In `debris-core/src/orphan/heuristic.rs`, remove the private `installed_bundle_ids` function and replace its call site with `crate::orphan::util::installed_bundle_ids`:

```rust
use crate::{dir_size, orphan::{OrphanItem, OrphanSource, util::installed_bundle_ids}};
use std::{
    fs,
    path::{Path, PathBuf},
};
```

Remove the entire `fn installed_bundle_ids(...)` block (it now lives in util.rs). The `scan_heuristic` function body stays identical, just update the import.

- [ ] **Step 5: Add pub mod util and pub mod containers placeholder to orphan/mod.rs**

`debris-core/src/orphan/mod.rs`:
```rust
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum OrphanSource {
    KnownDb,
    Heuristic,
    Containers,
}

#[derive(Debug, Clone)]
pub struct OrphanItem {
    pub name: String,
    pub paths: Vec<PathBuf>,
    pub total_size: u64,
    pub source: OrphanSource,
}

pub mod containers;
pub mod heuristic;
pub mod known;
pub mod util;
```

- [ ] **Step 6: Create containers.rs stub so it compiles**

`debris-core/src/orphan/containers.rs`:
```rust
use crate::orphan::OrphanItem;
use std::path::Path;

pub fn scan_containers(_home: &Path, _applications: &Path) -> Vec<OrphanItem> {
    vec![]
}
```

- [ ] **Step 7: Run all tests to verify nothing broke**

```bash
cargo test -p debris-core
```
Expected: All 16 tests pass.

- [ ] **Step 8: Commit**

```bash
git add debris-core/src/orphan/
git commit -m "refactor(core): extract installed_bundle_ids to shared util, add Containers OrphanSource"
```

---

### Task 2: Containers Orphan Scanner

**Files:**
- Modify: `debris-core/src/orphan/containers.rs`

- [ ] **Step 1: Write failing tests**

Replace the stub in `debris-core/src/orphan/containers.rs` with tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plist(path: &std::path::PathBuf, bundle_id: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>MCMMetadataIdentifier</key><string>{bundle_id}</string>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_no_containers_returns_empty() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(home.path().join("Library/Containers")).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_orphaned_container_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        let container = home.path().join("Library/Containers/ABC-UUID-123");
        fs::create_dir_all(&container).unwrap();
        write_plist(
            &container.join(".com.apple.containermanagerd.metadata.plist"),
            "com.example.vanished",
        );
        let result = scan_containers(home.path(), apps.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "com.example.vanished");
        assert_eq!(result[0].source, super::super::OrphanSource::Containers);
    }

    #[test]
    fn test_installed_container_not_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();

        // Create installed app with plist
        let app_dir = apps.path().join("MyApp.app/Contents");
        fs::create_dir_all(&app_dir).unwrap();
        let plist_content = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>CFBundleIdentifier</key><string>com.example.myapp</string>\
             </dict></plist>";
        fs::write(app_dir.join("Info.plist"), plist_content).unwrap();

        // Create container for that installed app
        let container = home.path().join("Library/Containers/UUID-123");
        fs::create_dir_all(&container).unwrap();
        write_plist(
            &container.join(".com.apple.containermanagerd.metadata.plist"),
            "com.example.myapp",
        );

        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty(), "installed app container should not be flagged");
    }

    #[test]
    fn test_orphaned_group_container_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(
            home.path().join("Library/Group Containers/group.com.example.gone"),
        ).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "group.com.example.gone");
    }

    #[test]
    fn test_apple_group_container_skipped() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        // Apple's own group containers — never orphaned
        fs::create_dir_all(
            home.path().join("Library/Group Containers/group.com.apple.notes"),
        ).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty(), "apple group containers should be skipped");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p debris-core orphan::containers
```
Expected: FAIL — `scan_containers` returns empty vec (stub).

- [ ] **Step 3: Implement scan_containers**

`debris-core/src/orphan/containers.rs`:
```rust
use crate::{
    dir_size,
    orphan::{util::installed_bundle_ids, OrphanItem, OrphanSource},
};
use std::{fs, path::Path};

pub fn scan_containers(home: &Path, applications: &Path) -> Vec<OrphanItem> {
    let installed = installed_bundle_ids(applications);
    let mut results = Vec::new();

    scan_sandboxed_containers(home, &installed, &mut results);
    scan_group_containers(home, &installed, &mut results);

    results
}

fn scan_sandboxed_containers(
    home: &Path,
    installed: &std::collections::HashSet<String>,
    results: &mut Vec<OrphanItem>,
) {
    let containers_dir = home.join("Library/Containers");
    let Ok(entries) = fs::read_dir(&containers_dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }

        let meta_plist = path.join(".com.apple.containermanagerd.metadata.plist");
        if !meta_plist.exists() { continue; }

        let Ok(val) = plist::from_file::<_, plist::Value>(&meta_plist) else { continue };
        let Some(bundle_id) = val
            .as_dictionary()
            .and_then(|d| d.get("MCMMetadataIdentifier"))
            .and_then(|v| v.as_string())
        else { continue };

        if installed.contains(&bundle_id.to_lowercase()) { continue; }

        let size = dir_size(&path);
        results.push(OrphanItem {
            name: bundle_id.to_string(),
            paths: vec![path],
            total_size: size,
            source: OrphanSource::Containers,
        });
    }
}

fn is_group_id(name: &str) -> bool {
    // Matches: group.com.x.y  or  TEAMID.com.x.y  or  com.x.y
    // Excludes Apple's own group containers
    let lower = name.to_lowercase();
    if lower.contains(".apple.") { return false; }
    let stripped = lower.strip_prefix("group.").unwrap_or(&lower);
    let parts: Vec<&str> = stripped.split('.').collect();
    parts.len() >= 2 && matches!(parts[0], "com" | "org" | "io" | "net" | "app" | "co")
}

fn scan_group_containers(
    home: &Path,
    installed: &std::collections::HashSet<String>,
    results: &mut Vec<OrphanItem>,
) {
    let group_dir = home.join("Library/Group Containers");
    let Ok(entries) = fs::read_dir(&group_dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !is_group_id(name) { continue; }

        // Extract the base bundle ID (strip group. prefix and team ID prefix)
        let base = name
            .strip_prefix("group.")
            .unwrap_or(name)
            .to_lowercase();

        // Check if any installed app matches this group container
        let is_installed = installed.iter().any(|id| {
            id == &base || base.starts_with(&format!("{id}.")) || id.starts_with(&format!("{base}."))
        });

        if is_installed { continue; }

        let size = dir_size(&path);
        results.push(OrphanItem {
            name: name.to_string(),
            paths: vec![path],
            total_size: size,
            source: OrphanSource::Containers,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plist(path: &std::path::PathBuf, bundle_id: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>MCMMetadataIdentifier</key><string>{bundle_id}</string>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_no_containers_returns_empty() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(home.path().join("Library/Containers")).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_orphaned_container_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        let container = home.path().join("Library/Containers/ABC-UUID-123");
        fs::create_dir_all(&container).unwrap();
        write_plist(
            &container.join(".com.apple.containermanagerd.metadata.plist"),
            "com.example.vanished",
        );
        let result = scan_containers(home.path(), apps.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "com.example.vanished");
        assert_eq!(result[0].source, OrphanSource::Containers);
    }

    #[test]
    fn test_installed_container_not_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        let app_dir = apps.path().join("MyApp.app/Contents");
        fs::create_dir_all(&app_dir).unwrap();
        let plist_content = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>CFBundleIdentifier</key><string>com.example.myapp</string>\
             </dict></plist>";
        fs::write(app_dir.join("Info.plist"), plist_content).unwrap();
        let container = home.path().join("Library/Containers/UUID-123");
        fs::create_dir_all(&container).unwrap();
        write_plist(
            &container.join(".com.apple.containermanagerd.metadata.plist"),
            "com.example.myapp",
        );
        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_orphaned_group_container_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(
            home.path().join("Library/Group Containers/group.com.example.gone"),
        ).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "group.com.example.gone");
    }

    #[test]
    fn test_apple_group_container_skipped() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(
            home.path().join("Library/Group Containers/group.com.apple.notes"),
        ).unwrap();
        let result = scan_containers(home.path(), apps.path());
        assert!(result.is_empty());
    }
}
```

- [ ] **Step 4: Wire containers into scanner.rs**

In `debris-core/src/scanner.rs`, add the containers scan after heuristic scan:

```rust
use crate::{
    dev_cache::{scan_dev_caches, DevCacheItem},
    orphan::{
        containers::scan_containers,
        heuristic::scan_heuristic,
        known::scan_known,
        OrphanItem,
    },
};
use std::{path::PathBuf, sync::mpsc, thread};
use debris_db::load_app_entries;

#[derive(Debug, Clone)]
pub enum ScanEvent {
    OrphanFound(OrphanItem),
    DevCacheFound(DevCacheItem),
    Done,
}

pub fn run_scan(home: PathBuf, applications: PathBuf) -> mpsc::Receiver<ScanEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let entries = load_app_entries();
        let mut seen_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

        for item in scan_known(&entries, &home, &applications) {
            for p in &item.paths { seen_paths.insert(p.clone()); }
            let _ = tx.send(ScanEvent::OrphanFound(item));
        }

        for item in scan_heuristic(&home, &applications) {
            if !item.paths.iter().any(|p| seen_paths.contains(p)) {
                for p in &item.paths { seen_paths.insert(p.clone()); }
                let _ = tx.send(ScanEvent::OrphanFound(item));
            }
        }

        for item in scan_containers(&home, &applications) {
            if !item.paths.iter().any(|p| seen_paths.contains(p)) {
                let _ = tx.send(ScanEvent::OrphanFound(item));
            }
        }

        for item in scan_dev_caches(&home) {
            let _ = tx.send(ScanEvent::DevCacheFound(item));
        }

        let _ = tx.send(ScanEvent::Done);
    });

    rx
}
```

- [ ] **Step 5: Run all tests**

```bash
cargo test -p debris-core
```
Expected: All tests pass (16 existing + 5 new container tests = 21 tests).

- [ ] **Step 6: Commit**

```bash
git add debris-core/src/orphan/
git commit -m "feat(core): add containers and group containers orphan scanner"
```

---

### Task 3: LaunchAgent Scanner

**Files:**
- Create: `debris-core/src/launch_agent.rs`
- Modify: `debris-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

`debris-core/src/launch_agent.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plist_with_program(path: &std::path::PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>Program</key><string>{program}</string>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    fn write_plist_with_args(path: &std::path::PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>ProgramArguments</key><array><string>{program}</string></array>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_no_agents_dir_returns_empty() {
        let home = tempdir().unwrap();
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_agent_with_missing_binary_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.missing.plist"),
            "/Applications/Vanished.app/Contents/MacOS/Vanished",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "com.example.missing");
    }

    #[test]
    fn test_agent_with_existing_binary_not_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        // Binary that actually exists
        let binary = home.path().join("real_binary");
        fs::write(&binary, b"fake binary").unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.real.plist"),
            binary.to_str().unwrap(),
        );
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty(), "agent pointing to existing binary should not be flagged");
    }

    #[test]
    fn test_agent_with_program_arguments_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_args(
            &agents_dir.join("com.example.args.plist"),
            "/usr/local/bin/vanished-daemon",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_agent_plist_size_included() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.sized.plist"),
            "/nonexistent/binary",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].size_bytes > 0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p debris-core launch_agent
```
Expected: FAIL — `scan_launch_agents` not defined.

- [ ] **Step 3: Implement launch_agent.rs**

`debris-core/src/launch_agent.rs`:
```rust
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct LaunchAgentItem {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

pub fn scan_launch_agents(home: &Path) -> Vec<LaunchAgentItem> {
    let agents_dir = home.join("Library/LaunchAgents");
    let Ok(entries) = fs::read_dir(&agents_dir) else { return vec![] };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "plist") { return None; }
            let binary = extract_binary_path(&path)?;
            if Path::new(&binary).exists() { return None; }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            Some(LaunchAgentItem { name, path, size_bytes })
        })
        .collect()
}

fn extract_binary_path(plist_path: &Path) -> Option<String> {
    let val = plist::from_file::<_, plist::Value>(plist_path).ok()?;
    let dict = val.as_dictionary()?;

    if let Some(program) = dict.get("Program").and_then(|v| v.as_string()) {
        return Some(program.to_string());
    }

    dict.get("ProgramArguments")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plist_with_program(path: &PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>Program</key><string>{program}</string>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    fn write_plist_with_args(path: &PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>ProgramArguments</key><array><string>{program}</string></array>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_no_agents_dir_returns_empty() {
        let home = tempdir().unwrap();
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_agent_with_missing_binary_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.missing.plist"),
            "/Applications/Vanished.app/Contents/MacOS/Vanished",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "com.example.missing");
    }

    #[test]
    fn test_agent_with_existing_binary_not_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        let binary = home.path().join("real_binary");
        fs::write(&binary, b"fake binary").unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.real.plist"),
            binary.to_str().unwrap(),
        );
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_agent_with_program_arguments_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_args(
            &agents_dir.join("com.example.args.plist"),
            "/usr/local/bin/vanished-daemon",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_agent_plist_size_included() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.sized.plist"),
            "/nonexistent/binary",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].size_bytes > 0);
    }
}
```

- [ ] **Step 4: Update scanner.rs to emit LaunchAgentFound events**

`debris-core/src/scanner.rs` — add `LaunchAgentFound` variant and wire it in:

```rust
use crate::{
    dev_cache::{scan_dev_caches, DevCacheItem},
    launch_agent::{scan_launch_agents, LaunchAgentItem},
    orphan::{
        containers::scan_containers,
        heuristic::scan_heuristic,
        known::scan_known,
        OrphanItem,
    },
};
use std::{path::PathBuf, sync::mpsc, thread};
use debris_db::load_app_entries;

#[derive(Debug, Clone)]
pub enum ScanEvent {
    OrphanFound(OrphanItem),
    DevCacheFound(DevCacheItem),
    LaunchAgentFound(LaunchAgentItem),
    Done,
}

pub fn run_scan(home: PathBuf, applications: PathBuf) -> mpsc::Receiver<ScanEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let entries = load_app_entries();
        let mut seen_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

        for item in scan_known(&entries, &home, &applications) {
            for p in &item.paths { seen_paths.insert(p.clone()); }
            let _ = tx.send(ScanEvent::OrphanFound(item));
        }

        for item in scan_heuristic(&home, &applications) {
            if !item.paths.iter().any(|p| seen_paths.contains(p)) {
                for p in &item.paths { seen_paths.insert(p.clone()); }
                let _ = tx.send(ScanEvent::OrphanFound(item));
            }
        }

        for item in scan_containers(&home, &applications) {
            if !item.paths.iter().any(|p| seen_paths.contains(p)) {
                let _ = tx.send(ScanEvent::OrphanFound(item));
            }
        }

        for item in scan_dev_caches(&home) {
            let _ = tx.send(ScanEvent::DevCacheFound(item));
        }

        for item in scan_launch_agents(&home) {
            let _ = tx.send(ScanEvent::LaunchAgentFound(item));
        }

        let _ = tx.send(ScanEvent::Done);
    });

    rx
}
```

- [ ] **Step 5: Update lib.rs exports**

`debris-core/src/lib.rs`:
```rust
pub mod cleaner;
pub mod dev_cache;
pub mod disk;
pub mod launch_agent;
pub mod orphan;
pub mod scanner;
pub mod size;

pub use cleaner::delete_path;
pub use dev_cache::{scan_dev_caches, DevCacheItem};
pub use disk::{get_disk_info, DiskInfo};
pub use launch_agent::{scan_launch_agents, LaunchAgentItem};
pub use orphan::{OrphanItem, OrphanSource};
pub use scanner::{run_scan, ScanEvent};
pub use size::dir_size;
```

- [ ] **Step 6: Run all tests**

```bash
cargo test -p debris-core
```
Expected: All 26 tests pass (21 existing + 5 new launch agent tests).

- [ ] **Step 7: Commit**

```bash
git add debris-core/src/launch_agent.rs debris-core/src/scanner.rs debris-core/src/lib.rs
git commit -m "feat(core): add launch agent orphan scanner"
```

---

### Task 4: Wire Launch Agents into App State + UI

**Files:**
- Modify: `debris-app/src/app.rs`
- Modify: `debris-app/src/ui/mod.rs`
- Create: `debris-app/src/ui/launch_agents.rs`
- Modify: `debris-app/src/ui/sidebar.rs`
- Modify: `debris-app/src/ui/overview.rs`

- [ ] **Step 1: Update app.rs — add launch_agents field and Section::LaunchAgents**

In `debris-app/src/app.rs`:

1. Add `LaunchAgentItem` to the import:
```rust
use debris_core::{DevCacheItem, DiskInfo, LaunchAgentItem, OrphanItem, ScanEvent};
```

2. Add `LaunchAgents` variant to `Section` enum:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Overview,
    Orphaned,
    DevCaches,
    LaunchAgents,
}
```

3. Add `launch_agents` field to `SweepApp`:
```rust
pub struct SweepApp {
    pub section: Section,
    pub disk_info: Option<DiskInfo>,
    pub orphans: Vec<OrphanItem>,
    pub dev_caches: Vec<DevCacheItem>,
    pub launch_agents: Vec<LaunchAgentItem>,
    pub selected: HashSet<usize>,
    pub scan_rx: Option<mpsc::Receiver<ScanEvent>>,
    pub scanning: bool,
    pub confirm_delete: bool,
}
```

4. Initialize `launch_agents` in `new()` and clear in `start_scan()`:
```rust
// In Self { ... }:
launch_agents: Vec::new(),

// In start_scan():
self.launch_agents.clear();
```

5. Drain `LaunchAgentFound` in `logic()`:
```rust
ScanEvent::LaunchAgentFound(item) => self.launch_agents.push(item),
```

6. Add match arm in `ui()`:
```rust
Section::LaunchAgents => crate::ui::launch_agents::draw_launch_agents(ui, self),
```

- [ ] **Step 2: Update ui/mod.rs**

`debris-app/src/ui/mod.rs`:
```rust
pub mod dev_caches;
pub mod launch_agents;
pub mod orphaned;
pub mod overview;
pub mod sidebar;

pub(crate) fn format_bytes(bytes: u64) -> String {
    // (keep existing implementation)
}
```

Read the current mod.rs first and just add `pub mod launch_agents;` to the existing file.

- [ ] **Step 3: Create launch_agents.rs panel**

`debris-app/src/ui/launch_agents.rs`:
```rust
use crate::app::SweepApp;
use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};

pub fn draw_launch_agents(ui: &mut Ui, app: &mut SweepApp) {
    ui.add_space(20.0);
    ui.heading(RichText::new("Launch Agents").color(Color32::WHITE));
    ui.add_space(4.0);
    ui.label(
        RichText::new("Background services from apps that are no longer installed")
            .color(Color32::from_gray(140)),
    );
    ui.add_space(16.0);

    if app.scanning && app.launch_agents.is_empty() {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.add_space(8.0);
            ui.label(RichText::new("Scanning…").color(Color32::from_gray(140)));
        });
        return;
    }

    if app.launch_agents.is_empty() {
        ui.add_space(60.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("✓ No orphaned launch agents found")
                    .color(Color32::from_gray(140))
                    .size(16.0),
            );
        });
        return;
    }

    let total_bytes: u64 = app.launch_agents.iter().map(|a| a.size_bytes).sum();
    ui.label(
        RichText::new(format!(
            "{} agents  ·  {}",
            app.launch_agents.len(),
            super::format_bytes(total_bytes)
        ))
        .color(Color32::from_gray(100))
        .size(12.0),
    );
    ui.add_space(8.0);
    ui.separator();

    let mut to_delete: Option<usize> = None;

    ScrollArea::vertical().show(ui, |ui| {
        for (i, agent) in app.launch_agents.iter().enumerate() {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&agent.name).color(Color32::WHITE).size(14.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn = egui::Button::new(
                                RichText::new("Delete").color(Color32::WHITE).size(12.0),
                            )
                            .fill(Color32::from_rgb(239, 68, 68))
                            .corner_radius(6.0)
                            .min_size(egui::vec2(70.0, 24.0));
                            if ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                to_delete = Some(i);
                            }
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(super::format_bytes(agent.size_bytes))
                                    .color(Color32::from_gray(160))
                                    .size(13.0),
                            );
                            ui.add_space(8.0);
                            egui::Frame::new()
                                .fill(Color32::from_rgb(40, 30, 10))
                                .corner_radius(4.0)
                                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("LaunchAgent")
                                            .color(Color32::from_rgb(234, 179, 8))
                                            .size(11.0),
                                    );
                                });
                        });
                    });
                });
            ui.separator();
        }
    });

    if let Some(idx) = to_delete {
        if let Some(item) = app.launch_agents.get(idx) {
            let _ = debris_core::delete_path(&item.path);
        }
        app.launch_agents.remove(idx);
    }
}
```

- [ ] **Step 4: Update sidebar.rs — add Launch Agents nav item**

In `debris-app/src/ui/sidebar.rs`, add a fourth `nav_item` call after Dev Caches:

```rust
nav_item(ui, app, Section::LaunchAgents, "Launch Agents", Some(app.launch_agents.len()));
```

Also update the import at the top to include `Section` (it should already be imported).

- [ ] **Step 5: Update overview.rs — add launch agents bar**

In `debris-app/src/ui/overview.rs`, in `draw_category_bars`, add a third bar for launch agents:

```rust
let launch_bytes: u64 = app.launch_agents.iter().map(|a| a.size_bytes).sum();
category_bar(ui, "Launch Agents", launch_bytes, total, Color32::from_rgb(234, 179, 8));
```

- [ ] **Step 6: Verify compilation**

```bash
cargo build -p debris-app
```
Expected: 0 errors, 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add debris-app/src/
git commit -m "feat(app): add Launch Agents panel and wire containers/launch agents into UI"
```

---

### Task 5: Final verification and push

- [ ] **Step 1: Run full test suite**

```bash
cargo test
```
Expected: All 26 debris-core tests + 3 debris-db tests = 29 tests pass.

- [ ] **Step 2: Run clippy**

```bash
cargo clippy -- -D warnings
```
Expected: 0 errors, 0 warnings.

- [ ] **Step 3: Run the app**

```bash
cargo run -p debris-app
```
Visually verify:
- Sidebar shows 4 nav items: Overview, Orphaned, Dev Caches, Launch Agents
- Orphaned section now shows Containers-source items (tagged differently or same "Orphaned" tag)
- Launch Agents section shows any orphaned plist files in ~/Library/LaunchAgents
- Overview bars include Launch Agents row

- [ ] **Step 4: Push to GitHub**

```bash
git push origin main
```

---

## Self-Review

**Spec coverage:**
- ✅ `~/Library/Containers` scanning → Task 2 (scan_containers, sandboxed)
- ✅ `~/Library/Group Containers` scanning → Task 2 (scan_group_containers)
- ✅ Apple group containers skipped → Task 2 (is_group_id skips `.apple.`)
- ✅ LaunchAgents with missing binary → Task 3 (scan_launch_agents)
- ✅ Both `Program` and `ProgramArguments` keys handled → Task 3
- ✅ UI panel for Launch Agents → Task 4
- ✅ Sidebar badge for Launch Agents → Task 4
- ✅ Overview breakdown updated → Task 4

**Placeholder scan:** No TBDs. All code blocks are complete.

**Type consistency:**
- `LaunchAgentItem { name: String, path: PathBuf, size_bytes: u64 }` — used in Tasks 3, 4 consistently
- `ScanEvent::LaunchAgentFound(LaunchAgentItem)` — defined in Task 3, drained in Task 4
- `OrphanSource::Containers` — defined in Task 1, used in Task 2
- `scan_containers(home: &Path, applications: &Path) -> Vec<OrphanItem>` — defined in Task 2, called in Task 3's scanner.rs

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

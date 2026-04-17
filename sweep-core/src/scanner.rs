use crate::{
    dev_cache::{scan_dev_caches, DevCacheItem},
    orphan::{
        heuristic::scan_heuristic,
        known::scan_known,
        OrphanItem,
    },
};
use std::{path::PathBuf, sync::mpsc, thread};
use sweep_db::load_app_entries;

#[derive(Debug, Clone)]
pub enum ScanEvent {
    OrphanFound(OrphanItem),
    DevCacheFound(DevCacheItem),
    Done,
}

pub fn run_scan(home: PathBuf, applications: PathBuf) -> mpsc::Receiver<ScanEvent> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        // Known-paths scan
        let entries = load_app_entries();
        for item in scan_known(&entries, &home, &applications) {
            let _ = tx.send(ScanEvent::OrphanFound(item));
        }

        // Heuristic scan
        for item in scan_heuristic(&home, &applications) {
            let _ = tx.send(ScanEvent::OrphanFound(item));
        }

        // Dev caches
        for item in scan_dev_caches(&home) {
            let _ = tx.send(ScanEvent::DevCacheFound(item));
        }

        let _ = tx.send(ScanEvent::Done);
    });

    rx
}

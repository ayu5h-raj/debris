use crate::orphan::OrphanItem;
use std::path::Path;

pub fn scan_heuristic(_home: &Path, _applications: &Path) -> Vec<OrphanItem> {
    vec![]
}

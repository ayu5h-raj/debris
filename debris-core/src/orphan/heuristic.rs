use crate::{dir_size, orphan::{util::installed_bundle_ids, OrphanItem, OrphanSource}};
use std::{
    fs,
    path::Path,
};

fn is_bundle_id(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2
        && matches!(parts[0], "com" | "org" | "io" | "net" | "app" | "co")
}

pub fn scan_heuristic(home: &Path, applications: &Path) -> Vec<OrphanItem> {
    let installed = installed_bundle_ids(applications);
    let support_dir = home.join("Library/Application Support");
    let Ok(entries) = fs::read_dir(&support_dir) else { return vec![] };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() { return None; }
            let name = path.file_name()?.to_str()?.to_string();
            if !is_bundle_id(&name) { return None; }
            if installed.contains(&name.to_lowercase()) { return None; }
            let size = dir_size(&path);
            Some(OrphanItem {
                name: name.clone(),
                paths: vec![path],
                total_size: size,
                source: OrphanSource::Heuristic,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_no_data_dirs_returns_empty() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(home.path().join("Library/Application Support")).unwrap();
        let result = scan_heuristic(home.path(), apps.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_bundle_id_dir_with_no_matching_app_flagged() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(
            home.path().join("Library/Application Support/com.example.vanished"),
        ).unwrap();
        let result = scan_heuristic(home.path(), apps.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source, OrphanSource::Heuristic);
    }

    #[test]
    fn test_non_bundle_id_dir_ignored() {
        let home = tempdir().unwrap();
        let apps = tempdir().unwrap();
        fs::create_dir_all(
            home.path().join("Library/Application Support/MyPlainApp"),
        ).unwrap();
        let result = scan_heuristic(home.path(), apps.path());
        assert!(result.is_empty(), "plain names are not bundle IDs");
    }
}

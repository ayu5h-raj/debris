use crate::{dir_size, orphan::{OrphanItem, OrphanSource}};
use std::{
    collections::HashSet,
    fs,
    path::Path,
};

fn is_bundle_id(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2
        && matches!(parts[0], "com" | "org" | "io" | "net" | "app" | "co")
}

fn installed_bundle_ids(applications: &Path) -> HashSet<String> {
    let mut ids = HashSet::new();
    let Ok(entries) = fs::read_dir(applications) else { return ids };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "app") {
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

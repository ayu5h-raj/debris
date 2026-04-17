use crate::{dir_size, orphan::{OrphanItem, OrphanSource}};
use std::path::{Path, PathBuf};
use debris_db::AppEntry;

pub fn scan_known(entries: &[AppEntry], home: &Path, applications: &Path) -> Vec<OrphanItem> {
    entries
        .iter()
        .filter_map(|entry| {
            if applications.join(&entry.app_bundle).exists() {
                return None;
            }
            let existing: Vec<PathBuf> = entry
                .paths
                .iter()
                .map(|p| home.join(p))
                .filter(|p| p.exists())
                .collect();
            if existing.is_empty() {
                return None;
            }
            let total_size = existing.iter().map(|p| dir_size(p)).sum();
            Some(OrphanItem {
                name: entry.name.clone(),
                paths: existing,
                total_size,
                source: OrphanSource::KnownDb,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use debris_db::AppEntry;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_installed_app_not_flagged() {
        let apps_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        fs::create_dir(apps_dir.path().join("MyApp.app")).unwrap();
        fs::create_dir_all(home_dir.path().join("Library/Application Support/MyApp")).unwrap();

        let entry = AppEntry {
            name: "MyApp".into(),
            app_bundle: "MyApp.app".into(),
            paths: vec!["Library/Application Support/MyApp".into()],
        };
        let result = scan_known(&[entry], home_dir.path(), apps_dir.path());
        assert!(result.is_empty(), "installed app should not be flagged");
    }

    #[test]
    fn test_uninstalled_app_flagged() {
        let apps_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();
        fs::create_dir_all(home_dir.path().join("Library/Application Support/MyApp")).unwrap();

        let entry = AppEntry {
            name: "MyApp".into(),
            app_bundle: "MyApp.app".into(),
            paths: vec!["Library/Application Support/MyApp".into()],
        };
        let result = scan_known(&[entry], home_dir.path(), apps_dir.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "MyApp");
        assert_eq!(result[0].source, super::super::OrphanSource::KnownDb);
    }

    #[test]
    fn test_no_existing_paths_not_flagged() {
        let apps_dir = tempdir().unwrap();
        let home_dir = tempdir().unwrap();

        let entry = AppEntry {
            name: "Ghost".into(),
            app_bundle: "Ghost.app".into(),
            paths: vec!["Library/Application Support/Ghost".into()],
        };
        let result = scan_known(&[entry], home_dir.path(), apps_dir.path());
        assert!(result.is_empty());
    }
}

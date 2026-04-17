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

        let base = name
            .strip_prefix("group.")
            .unwrap_or(name)
            .to_lowercase();

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

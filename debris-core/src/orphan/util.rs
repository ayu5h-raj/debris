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

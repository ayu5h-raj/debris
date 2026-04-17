use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppEntry {
    pub name: String,
    pub app_bundle: String,
    pub paths: Vec<String>,
}

#[derive(Deserialize)]
struct AppDatabase {
    apps: Vec<AppEntry>,
}

const APPS_TOML: &str = include_str!("../data/apps.toml");

pub fn load_app_entries() -> Vec<AppEntry> {
    let db: AppDatabase = toml::from_str(APPS_TOML).expect("invalid apps.toml");
    db.apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_returns_entries() {
        let entries = load_app_entries();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_google_chrome_entry_exists() {
        let entries = load_app_entries();
        let chrome = entries.iter().find(|e| e.name == "Google Chrome");
        assert!(chrome.is_some());
        assert!(!chrome.unwrap().paths.is_empty());
    }
}

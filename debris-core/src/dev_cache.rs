use crate::dir_size;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DevCacheItem {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

struct CacheDef {
    name: &'static str,
    relative_path: &'static str,
}

const CACHES: &[CacheDef] = &[
    CacheDef { name: "npm",                    relative_path: ".npm" },
    CacheDef { name: "yarn",                   relative_path: "Library/Caches/Yarn" },
    CacheDef { name: "uv",                     relative_path: ".cache/uv" },
    CacheDef { name: "pip",                    relative_path: "Library/Caches/pip" },
    CacheDef { name: "Go modules",             relative_path: "go/pkg/mod" },
    CacheDef { name: "Cargo registry",         relative_path: ".cargo/registry" },
    CacheDef { name: "Puppeteer",              relative_path: ".cache/puppeteer" },
    CacheDef { name: "node-gyp",               relative_path: "Library/Caches/node-gyp" },
    CacheDef { name: "Docker buildx",          relative_path: ".docker/buildx" },
    CacheDef { name: "Docker Desktop logs",    relative_path: "Library/Containers/com.docker.docker/Data/log" },
    CacheDef { name: "Docker Desktop support", relative_path: "Library/Application Support/Docker Desktop" },
];

pub fn scan_dev_caches(home: &Path) -> Vec<DevCacheItem> {
    CACHES
        .iter()
        .filter_map(|def| {
            let path = home.join(def.relative_path);
            if !path.exists() { return None; }
            let size_bytes = dir_size(&path);
            if size_bytes == 0 { return None; }
            Some(DevCacheItem { name: def.name.to_string(), path, size_bytes })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_no_caches_returns_empty() {
        let home = tempdir().unwrap();
        let result = scan_dev_caches(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_npm_cache_detected() {
        let home = tempdir().unwrap();
        fs::create_dir_all(home.path().join(".npm/_cacache")).unwrap();
        fs::write(home.path().join(".npm/_cacache/data.bin"), vec![0u8; 1024]).unwrap();
        let result = scan_dev_caches(home.path());
        let npm = result.iter().find(|c| c.name == "npm");
        assert!(npm.is_some());
        assert!(npm.unwrap().size_bytes >= 1024);
    }
}

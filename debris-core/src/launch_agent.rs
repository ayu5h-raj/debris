use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone)]
pub struct LaunchAgentItem {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}

pub fn scan_launch_agents(home: &Path) -> Vec<LaunchAgentItem> {
    let agents_dir = home.join("Library/LaunchAgents");
    let Ok(entries) = fs::read_dir(&agents_dir) else { return vec![] };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "plist") { return None; }
            let binary = extract_binary_path(&path)?;
            if Path::new(&binary).exists() { return None; }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            Some(LaunchAgentItem { name, path, size_bytes })
        })
        .collect()
}

fn extract_binary_path(plist_path: &Path) -> Option<String> {
    let val = plist::from_file::<_, plist::Value>(plist_path).ok()?;
    let dict = val.as_dictionary()?;

    if let Some(program) = dict.get("Program").and_then(|v| v.as_string()) {
        return Some(program.to_string());
    }

    dict.get("ProgramArguments")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_plist_with_program(path: &PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>Program</key><string>{program}</string>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    fn write_plist_with_args(path: &PathBuf, program: &str) {
        let content = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
             \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\
             <key>Label</key><string>com.example.agent</string>\
             <key>ProgramArguments</key><array><string>{program}</string></array>\
             </dict></plist>"
        );
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_no_agents_dir_returns_empty() {
        let home = tempdir().unwrap();
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_agent_with_missing_binary_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.missing.plist"),
            "/Applications/Vanished.app/Contents/MacOS/Vanished",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "com.example.missing");
    }

    #[test]
    fn test_agent_with_existing_binary_not_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        let binary = home.path().join("real_binary");
        fs::write(&binary, b"fake binary").unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.real.plist"),
            binary.to_str().unwrap(),
        );
        let result = scan_launch_agents(home.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_agent_with_program_arguments_flagged() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_args(
            &agents_dir.join("com.example.args.plist"),
            "/usr/local/bin/vanished-daemon",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_agent_plist_size_included() {
        let home = tempdir().unwrap();
        let agents_dir = home.path().join("Library/LaunchAgents");
        fs::create_dir_all(&agents_dir).unwrap();
        write_plist_with_program(
            &agents_dir.join("com.example.sized.plist"),
            "/nonexistent/binary",
        );
        let result = scan_launch_agents(home.path());
        assert_eq!(result.len(), 1);
        assert!(result[0].size_bytes > 0);
    }
}

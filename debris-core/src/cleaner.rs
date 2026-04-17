use std::{fs, io, path::Path};

pub fn delete_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_delete_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, b"data").unwrap();
        assert!(file.exists());
        delete_path(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_delete_directory() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("file.txt"), b"data").unwrap();
        delete_path(&sub).unwrap();
        assert!(!sub.exists());
    }
}

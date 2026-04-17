use nix::sys::statvfs::statvfs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

pub fn get_disk_info(path: &Path) -> io::Result<DiskInfo> {
    let stat = statvfs(path)
        .map_err(io::Error::other)?;
    let block_size = stat.block_size() as u64;
    let total = stat.blocks() as u64 * block_size;
    let free = stat.blocks_available() as u64 * block_size;
    let used = total.saturating_sub(free);
    Ok(DiskInfo { total_bytes: total, used_bytes: used, free_bytes: free })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_disk_info_root() {
        let info = get_disk_info(Path::new("/")).unwrap();
        assert!(info.total_bytes > 0);
        assert!(info.used_bytes > 0);
        assert!(info.free_bytes > 0);
        assert_eq!(info.total_bytes, info.used_bytes + info.free_bytes);
    }
}

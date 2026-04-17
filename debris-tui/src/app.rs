use debris_core::{
    get_disk_info, run_scan, delete_path,
    DevCacheItem, DiskInfo, LaunchAgentItem, OrphanItem, ScanEvent,
};
use std::{collections::HashSet, sync::mpsc};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Overview,
    Orphaned,
    DevCaches,
    LaunchAgents,
}

#[derive(Debug)]
pub enum ConfirmAction {
    DeleteOrphans,
    ClearCache(usize),
    DeleteAgent(usize),
}

pub struct TuiApp {
    pub tab: Tab,
    pub disk_info: Option<DiskInfo>,
    pub orphans: Vec<OrphanItem>,
    pub dev_caches: Vec<DevCacheItem>,
    pub launch_agents: Vec<LaunchAgentItem>,
    pub scanning: bool,
    pub scan_rx: Option<mpsc::Receiver<ScanEvent>>,
    pub orphan_cursor: usize,
    pub cache_cursor: usize,
    pub agent_cursor: usize,
    pub selected: HashSet<usize>,
    pub confirm: Option<ConfirmAction>,
    pub(crate) auto_selected: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let applications = std::path::PathBuf::from("/Applications");
        let disk_info = get_disk_info(&home).ok();
        let scan_rx = run_scan(home, applications);
        Self {
            tab: Tab::Overview,
            disk_info,
            orphans: Vec::new(),
            dev_caches: Vec::new(),
            launch_agents: Vec::new(),
            scanning: true,
            scan_rx: Some(scan_rx),
            orphan_cursor: 0,
            cache_cursor: 0,
            agent_cursor: 0,
            selected: HashSet::new(),
            confirm: None,
            auto_selected: false,
        }
    }

    pub fn start_scan(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        let applications = std::path::PathBuf::from("/Applications");
        self.disk_info = get_disk_info(&home).ok();
        self.orphans.clear();
        self.dev_caches.clear();
        self.launch_agents.clear();
        self.selected.clear();
        self.confirm = None;
        self.auto_selected = false;
        self.orphan_cursor = 0;
        self.cache_cursor = 0;
        self.agent_cursor = 0;
        self.scanning = true;
        self.scan_rx = Some(run_scan(home, applications));
    }

    pub fn tick(&mut self) {
        let Some(rx) = &self.scan_rx else { return };
        let mut done = false;
        while let Ok(event) = rx.try_recv() {
            match event {
                ScanEvent::OrphanFound(item) => self.orphans.push(item),
                ScanEvent::DevCacheFound(item) => self.dev_caches.push(item),
                ScanEvent::LaunchAgentFound(item) => self.launch_agents.push(item),
                ScanEvent::Done => done = true,
            }
        }
        if done {
            self.scanning = false;
            self.scan_rx = None;
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = match self.tab {
            Tab::Overview => Tab::Orphaned,
            Tab::Orphaned => Tab::DevCaches,
            Tab::DevCaches => Tab::LaunchAgents,
            Tab::LaunchAgents => Tab::Overview,
        };
    }

    pub fn cursor_down(&mut self) {
        match self.tab {
            Tab::Orphaned => {
                if !self.orphans.is_empty() {
                    self.orphan_cursor = (self.orphan_cursor + 1).min(self.orphans.len() - 1);
                }
            }
            Tab::DevCaches => {
                if !self.dev_caches.is_empty() {
                    self.cache_cursor = (self.cache_cursor + 1).min(self.dev_caches.len() - 1);
                }
            }
            Tab::LaunchAgents => {
                if !self.launch_agents.is_empty() {
                    self.agent_cursor = (self.agent_cursor + 1).min(self.launch_agents.len() - 1);
                }
            }
            Tab::Overview => {}
        }
    }

    pub fn cursor_up(&mut self) {
        match self.tab {
            Tab::Orphaned => {
                self.orphan_cursor = self.orphan_cursor.saturating_sub(1);
            }
            Tab::DevCaches => {
                self.cache_cursor = self.cache_cursor.saturating_sub(1);
            }
            Tab::LaunchAgents => {
                self.agent_cursor = self.agent_cursor.saturating_sub(1);
            }
            Tab::Overview => {}
        }
    }

    pub fn toggle_select(&mut self) {
        self.auto_selected = false;
        if self.tab != Tab::Orphaned || self.orphans.is_empty() {
            return;
        }
        let cursor = self.orphan_cursor;
        if self.selected.contains(&cursor) {
            self.selected.remove(&cursor);
        } else {
            self.selected.insert(cursor);
        }
    }

    pub fn request_delete(&mut self) {
        match self.tab {
            Tab::Orphaned => {
                if !self.selected.is_empty() {
                    self.confirm = Some(ConfirmAction::DeleteOrphans);
                } else if !self.orphans.is_empty() {
                    self.selected.insert(self.orphan_cursor);
                    self.auto_selected = true;
                    self.confirm = Some(ConfirmAction::DeleteOrphans);
                }
            }
            Tab::DevCaches => {
                if !self.dev_caches.is_empty() {
                    self.confirm = Some(ConfirmAction::ClearCache(self.cache_cursor));
                }
            }
            Tab::LaunchAgents => {
                if !self.launch_agents.is_empty() {
                    self.confirm = Some(ConfirmAction::DeleteAgent(self.agent_cursor));
                }
            }
            Tab::Overview => {}
        }
    }

    pub fn confirm_action(&mut self) {
        match self.confirm.take() {
            Some(ConfirmAction::DeleteOrphans) => {
                let mut indices: Vec<usize> = self.selected.iter().cloned().collect();
                // Sort descending so each removal doesn't shift indices of items still to be removed.
                indices.sort_unstable_by(|a, b| b.cmp(a));
                for idx in &indices {
                    if let Some(item) = self.orphans.get(*idx) {
                        for path in &item.paths {
                            let _ = delete_path(path);
                        }
                    }
                }
                for idx in &indices {
                    if *idx < self.orphans.len() {
                        self.orphans.remove(*idx);
                    }
                }
                self.selected.clear();
                self.auto_selected = false;
                self.orphan_cursor = self.orphan_cursor.min(self.orphans.len().saturating_sub(1));
            }
            Some(ConfirmAction::ClearCache(idx)) => {
                if let Some(item) = self.dev_caches.get(idx) {
                    let _ = delete_path(&item.path);
                }
                if idx < self.dev_caches.len() {
                    self.dev_caches.remove(idx);
                }
                self.cache_cursor = self.cache_cursor.min(self.dev_caches.len().saturating_sub(1));
            }
            Some(ConfirmAction::DeleteAgent(idx)) => {
                if let Some(item) = self.launch_agents.get(idx) {
                    let _ = delete_path(&item.path);
                }
                if idx < self.launch_agents.len() {
                    self.launch_agents.remove(idx);
                }
                self.agent_cursor = self.agent_cursor.min(self.launch_agents.len().saturating_sub(1));
            }
            None => {}
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
        if self.auto_selected {
            self.selected.clear();
            self.auto_selected = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_app_starts_scanning() {
        let app = TuiApp::new();
        assert!(app.scanning);
        assert!(app.scan_rx.is_some());
    }

    #[test]
    fn test_tab_cycles() {
        let mut app = TuiApp::new();
        app.tab = Tab::Overview;
        app.next_tab();
        assert_eq!(app.tab, Tab::Orphaned);
        app.next_tab();
        assert_eq!(app.tab, Tab::DevCaches);
        app.next_tab();
        assert_eq!(app.tab, Tab::LaunchAgents);
        app.next_tab();
        assert_eq!(app.tab, Tab::Overview);
    }

    #[test]
    fn test_cursor_down_clamps() {
        let mut app = TuiApp::new();
        app.tab = Tab::Orphaned;
        app.orphans = vec![
            OrphanItem {
                name: "a".into(),
                paths: vec![],
                total_size: 100,
                source: debris_core::OrphanSource::Heuristic,
            },
        ];
        app.cursor_down();
        assert_eq!(app.orphan_cursor, 0);
    }

    #[test]
    fn test_toggle_select() {
        let mut app = TuiApp::new();
        app.tab = Tab::Orphaned;
        app.orphans = vec![OrphanItem {
            name: "a".into(),
            paths: vec![],
            total_size: 0,
            source: debris_core::OrphanSource::Heuristic,
        }];
        app.toggle_select();
        assert!(app.selected.contains(&0));
        app.toggle_select();
        assert!(!app.selected.contains(&0));
    }
}

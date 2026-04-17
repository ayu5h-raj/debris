use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum OrphanSource {
    KnownDb,
    Heuristic,
}

#[derive(Debug, Clone)]
pub struct OrphanItem {
    pub name: String,
    pub paths: Vec<PathBuf>,
    pub total_size: u64,
    pub source: OrphanSource,
}

pub mod known;
pub mod heuristic;

use eframe::egui::{self, Color32, Style, Visuals};
use std::collections::HashSet;
use std::sync::mpsc;
use debris_core::{DevCacheItem, DiskInfo, OrphanItem, ScanEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Overview,
    Orphaned,
    DevCaches,
}

pub struct SweepApp {
    pub section: Section,
    pub disk_info: Option<DiskInfo>,
    pub orphans: Vec<OrphanItem>,
    pub dev_caches: Vec<DevCacheItem>,
    pub selected: HashSet<usize>,
    pub scan_rx: Option<mpsc::Receiver<ScanEvent>>,
    pub scanning: bool,
    pub confirm_delete: bool,
}

impl SweepApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_theme(&cc.egui_ctx);
        let mut app = Self {
            section: Section::Overview,
            disk_info: None,
            orphans: Vec::new(),
            dev_caches: Vec::new(),
            selected: HashSet::new(),
            scan_rx: None,
            scanning: false,
            confirm_delete: false,
        };
        app.start_scan();
        app
    }

    pub fn start_scan(&mut self) {
        let home = dirs::home_dir().unwrap_or_default();
        let applications = std::path::PathBuf::from("/Applications");
        self.disk_info = debris_core::get_disk_info(&home).ok();
        self.orphans.clear();
        self.dev_caches.clear();
        self.selected.clear();
        self.confirm_delete = false;
        self.scanning = true;
        self.scan_rx = Some(debris_core::run_scan(home, applications));
    }
}

fn setup_theme(ctx: &egui::Context) {
    let mut style = Style {
        visuals: Visuals::dark(),
        ..Default::default()
    };
    style.visuals.panel_fill = Color32::from_rgb(20, 20, 20);
    style.visuals.window_fill = Color32::from_rgb(20, 20, 20);
    style.visuals.faint_bg_color = Color32::from_rgb(26, 26, 26);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 30);
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 35);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(59, 130, 246);
    ctx.set_global_style(style);
}

impl eframe::App for SweepApp {
    /// Drain the scan channel and request repaints while scanning.
    /// Called before each `ui()` call (and also when the window is hidden but repaint was requested).
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.scan_rx {
            let mut done = false;
            while let Ok(event) = rx.try_recv() {
                match event {
                    ScanEvent::OrphanFound(item) => self.orphans.push(item),
                    ScanEvent::DevCacheFound(item) => self.dev_caches.push(item),
                    ScanEvent::Done => {
                        done = true;
                    }
                }
            }
            if done {
                self.scanning = false;
                self.scan_rx = None;
            } else if self.scanning {
                ctx.request_repaint();
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("sidebar")
            .exact_size(180.0)
            .show_inside(ui, |ui| {
                crate::ui::sidebar::draw_sidebar(ui, self);
            });

        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            match self.section {
                Section::Overview => crate::ui::overview::draw_overview(ui, self),
                Section::Orphaned => crate::ui::orphaned::draw_orphaned(ui, self),
                Section::DevCaches => crate::ui::dev_caches::draw_dev_caches(ui, self),
            }
        });
    }
}

use crate::app::SweepApp;
use eframe::egui::{self, Color32, Frame, RichText, Sense, Stroke, Vec2};

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

pub fn draw_overview(ui: &mut egui::Ui, app: &SweepApp) {
    Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 24))
        .show(ui, |ui| {
            ui.label(RichText::new("Storage Overview").size(22.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Disk usage and cleanup opportunities on this Mac.")
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(20.0);

            match &app.disk_info {
                None => {
                    ui.spinner();
                    ui.label(RichText::new("Loading…").color(Color32::from_gray(150)));
                }
                Some(disk) => {
                    let total = disk.total_bytes;
                    let used = disk.used_bytes;
                    let free = disk.free_bytes;

                    // Disk donut + stats row
                    ui.horizontal(|ui| {
                        // Donut chart 120x120
                        let (response, painter) =
                            ui.allocate_painter(Vec2::splat(120.0), Sense::hover());
                        let rect = response.rect;
                        let center = rect.center();
                        let outer_r = 54.0_f32;
                        let inner_r = 34.0_f32;
                        let stroke_w = outer_r - inner_r;
                        let mid_r = (outer_r + inner_r) / 2.0;

                        // Background ring (free space)
                        painter.circle_stroke(
                            center,
                            mid_r,
                            Stroke::new(stroke_w, Color32::from_gray(40)),
                        );

                        // Used arc — approximate with line segments
                        if total > 0 {
                            let used_frac = (used as f64 / total as f64) as f32;
                            let segments = 120usize;
                            let start_angle = -std::f32::consts::FRAC_PI_2; // top
                            let end_angle = start_angle + used_frac * std::f32::consts::TAU;

                            let mut prev = center
                                + Vec2::new(start_angle.cos(), start_angle.sin()) * mid_r;

                            for i in 1..=segments {
                                let t = i as f32 / segments as f32;
                                let angle = start_angle + t * (end_angle - start_angle);
                                let pt = center + Vec2::new(angle.cos(), angle.sin()) * mid_r;
                                painter.line_segment(
                                    [prev, pt],
                                    Stroke::new(stroke_w, Color32::from_rgb(59, 130, 246)),
                                );
                                prev = pt;
                            }
                        }

                        // Center text
                        if total > 0 {
                            let pct = (used as f64 / total as f64 * 100.0) as u32;
                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                format!("{}%", pct),
                                egui::FontId::proportional(13.0),
                                Color32::WHITE,
                            );
                        }

                        ui.add_space(16.0);

                        // Stats column
                        ui.vertical(|ui| {
                            ui.add_space(16.0);
                            stat_row(ui, "Used", &format_bytes(used), Color32::from_rgb(59, 130, 246));
                            ui.add_space(8.0);
                            stat_row(ui, "Free", &format_bytes(free), Color32::from_gray(150));
                            ui.add_space(8.0);
                            stat_row(ui, "Total", &format_bytes(total), Color32::from_gray(200));
                        });
                    });

                    ui.add_space(24.0);

                    // Category bars
                    ui.label(
                        RichText::new("Cleanup Opportunities")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(10.0);

                    let orphan_bytes: u64 = app.orphans.iter().map(|o| o.total_size).sum();
                    let cache_bytes: u64 = app.dev_caches.iter().map(|c| c.size_bytes).sum();

                    if total > 0 {
                        category_bar(
                            ui,
                            "Orphaned App Data",
                            orphan_bytes,
                            total,
                            Color32::from_rgb(239, 68, 68),
                        );
                        ui.add_space(8.0);
                        category_bar(
                            ui,
                            "Dev Caches",
                            cache_bytes,
                            total,
                            Color32::from_rgb(234, 179, 8),
                        );
                    }
                }
            }
        });
}

fn stat_row(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal(|ui| {
        // Colour dot
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.add_space(4.0);
        ui.label(RichText::new(label).color(Color32::from_gray(160)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).color(Color32::WHITE).strong());
        });
    });
}

fn category_bar(ui: &mut egui::Ui, label: &str, bytes: u64, total: u64, color: Color32) {
    ui.label(RichText::new(label).color(Color32::from_gray(200)));
    ui.add_space(4.0);

    let bar_height = 8.0_f32;
    let available_w = ui.available_width().min(400.0);
    let (bar_rect, _) =
        ui.allocate_exact_size(Vec2::new(available_w, bar_height), Sense::hover());

    // Background
    ui.painter().rect_filled(bar_rect, 4.0, Color32::from_gray(40));

    // Filled portion
    let frac = if total > 0 {
        (bytes as f64 / total as f64) as f32
    } else {
        0.0
    };
    let filled_w = (frac * available_w).max(0.0);
    if filled_w > 0.0 {
        let filled_rect = egui::Rect::from_min_size(bar_rect.min, Vec2::new(filled_w, bar_height));
        ui.painter().rect_filled(filled_rect, 4.0, color);
    }

    ui.add_space(2.0);
    ui.label(
        RichText::new(format_bytes(bytes))
            .size(11.0)
            .color(Color32::from_gray(150)),
    );
}

use crate::app::SweepApp;
use eframe::egui::{self, Color32, Frame, RichText, Stroke};

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

pub fn draw_orphaned(ui: &mut egui::Ui, app: &mut SweepApp) {
    Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 24))
        .show(ui, |ui| {
            ui.label(RichText::new("Orphaned Data").size(22.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Leftover files from uninstalled applications.")
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(16.0);

            if app.scanning && app.orphans.is_empty() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("  Scanning…").color(Color32::from_gray(150)));
                });
                return;
            }

            if app.orphans.is_empty() {
                ui.label(
                    RichText::new("✓ No orphaned data found").color(Color32::from_rgb(74, 222, 128)),
                );
                return;
            }

            // Header: count + total size
            let total_bytes: u64 = app.orphans.iter().map(|o| o.total_size).sum();
            let count = app.orphans.len();
            ui.label(
                RichText::new(format!("{} items — {}", count, format_bytes(total_bytes)))
                    .color(Color32::from_gray(180)),
            );
            ui.add_space(10.0);

            // Inline confirmation banner
            if app.confirm_delete {
                let sel_count = app.selected.len();
                let sel_bytes: u64 = app
                    .selected
                    .iter()
                    .filter_map(|&i| app.orphans.get(i))
                    .map(|o| o.total_size)
                    .sum();

                Frame::new()
                    .fill(Color32::from_rgb(30, 20, 20))
                    .corner_radius(6.0)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                    .inner_margin(egui::Margin::symmetric(12, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Delete {} items · {}?",
                                    sel_count,
                                    format_bytes(sel_bytes)
                                ))
                                .color(Color32::WHITE),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let del_btn = egui::Button::new(
                                        RichText::new("Delete").color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_rgb(220, 38, 38))
                                    .stroke(Stroke::NONE);

                                    if ui.add(del_btn).clicked() {
                                        // Collect indices in descending order to avoid shift
                                        let mut indices: Vec<usize> =
                                            app.selected.iter().cloned().collect();
                                        indices.sort_unstable_by(|a, b| b.cmp(a));
                                        for idx in &indices {
                                            if let Some(item) = app.orphans.get(*idx) {
                                                for path in &item.paths {
                                                    let _ = sweep_core::delete_path(path);
                                                }
                                            }
                                        }
                                        for idx in &indices {
                                            if *idx < app.orphans.len() {
                                                app.orphans.remove(*idx);
                                            }
                                        }
                                        app.selected.clear();
                                        app.confirm_delete = false;
                                    }

                                    ui.add_space(8.0);

                                    let cancel_btn = egui::Button::new(
                                        RichText::new("Cancel").color(Color32::WHITE),
                                    )
                                    .fill(Color32::from_gray(50))
                                    .stroke(Stroke::NONE);
                                    if ui.add(cancel_btn).clicked() {
                                        app.confirm_delete = false;
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(10.0);
            }

            // Scroll list
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Collect actions to apply after the loop (borrow checker)
                let mut to_delete_single: Option<usize> = None;
                let mut toggle_selected: Option<usize> = None;

                let orphans_snapshot: Vec<_> = app
                    .orphans
                    .iter()
                    .enumerate()
                    .map(|(i, o)| (i, o.name.clone(), o.total_size))
                    .collect();

                for (idx, name, size) in &orphans_snapshot {
                    let idx = *idx;
                    Frame::new()
                        .fill(Color32::from_rgb(24, 24, 24))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Checkbox
                                let mut checked = app.selected.contains(&idx);
                                if ui.checkbox(&mut checked, "").changed() {
                                    toggle_selected = Some(idx);
                                }

                                // Name
                                ui.label(RichText::new(name).color(Color32::WHITE));

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Trash button
                                        let trash = egui::Button::new(
                                            RichText::new("🗑").size(14.0),
                                        )
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE);
                                        if ui.add(trash).clicked() {
                                            to_delete_single = Some(idx);
                                        }

                                        ui.add_space(8.0);

                                        // Red "Orphaned" tag
                                        Frame::new()
                                            .fill(Color32::from_rgb(127, 29, 29))
                                            .corner_radius(4.0)
                                            .inner_margin(egui::Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new("Orphaned")
                                                        .color(Color32::from_rgb(252, 165, 165))
                                                        .size(11.0),
                                                );
                                            });

                                        ui.add_space(8.0);

                                        // Size
                                        ui.label(
                                            RichText::new(format_bytes(*size))
                                                .color(Color32::from_gray(180)),
                                        );
                                    },
                                );
                            });
                        });
                    ui.add_space(4.0);
                }

                // Apply deferred mutations
                if let Some(idx) = toggle_selected {
                    if app.selected.contains(&idx) {
                        app.selected.remove(&idx);
                    } else {
                        app.selected.insert(idx);
                    }
                }

                if let Some(idx) = to_delete_single {
                    if let Some(item) = app.orphans.get(idx) {
                        for path in &item.paths {
                            let _ = sweep_core::delete_path(path);
                        }
                    }
                    app.orphans.remove(idx);
                    app.selected.remove(&idx);
                    // Shift down all selected indices above the deleted one
                    let above: Vec<usize> = app.selected.iter().filter(|&&i| i > idx).cloned().collect();
                    for i in above {
                        app.selected.remove(&i);
                        app.selected.insert(i - 1);
                    }
                }
            });

            // "Delete N Selected" button at bottom
            if !app.selected.is_empty() && !app.confirm_delete {
                ui.add_space(12.0);
                let n = app.selected.len();
                let del_bytes: u64 = app
                    .selected
                    .iter()
                    .filter_map(|&i| app.orphans.get(i))
                    .map(|o| o.total_size)
                    .sum();

                let del_btn = egui::Button::new(
                    RichText::new(format!("Delete {} Selected ({})", n, format_bytes(del_bytes)))
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(Color32::from_rgb(220, 38, 38))
                .stroke(Stroke::NONE)
                .min_size(egui::Vec2::new(200.0, 32.0));

                if ui.add(del_btn).clicked() {
                    app.confirm_delete = true;
                }
            }
        });
}

use crate::app::SweepApp;
use eframe::egui::{self, Color32, Frame, RichText, Stroke};

pub fn draw_dev_caches(ui: &mut egui::Ui, app: &mut SweepApp) {
    Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 24))
        .show(ui, |ui| {
            ui.label(RichText::new("Dev Caches").size(22.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Package manager and tool caches — safe to delete.")
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(16.0);

            if app.dev_caches.is_empty() {
                ui.label(
                    RichText::new("No dev caches found").color(Color32::from_gray(150)),
                );
                return;
            }

            let total_bytes: u64 = app.dev_caches.iter().map(|c| c.size_bytes).sum();
            let count = app.dev_caches.len();
            ui.label(
                RichText::new(format!("{} caches — {}", count, super::format_bytes(total_bytes)))
                    .color(Color32::from_gray(180)),
            );
            ui.add_space(10.0);

            // Inline confirmation banner
            if let Some(pending_idx) = app.confirm_clear_cache {
                if let Some(cache) = app.dev_caches.get(pending_idx) {
                    let name = cache.name.clone();
                    let size = cache.size_bytes;

                    Frame::new()
                        .fill(Color32::from_rgb(30, 20, 20))
                        .corner_radius(6.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "Clear \"{}\" · {}?",
                                        name,
                                        super::format_bytes(size)
                                    ))
                                    .color(Color32::WHITE),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let del_btn = egui::Button::new(
                                            RichText::new("Clear").color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(220, 38, 38))
                                        .stroke(Stroke::NONE);

                                        if ui.add(del_btn).clicked() {
                                            if let Some(item) = app.dev_caches.get(pending_idx) {
                                                let _ = debris_core::delete_path(&item.path);
                                            }
                                            app.dev_caches.remove(pending_idx);
                                            app.confirm_clear_cache = None;
                                        }

                                        ui.add_space(8.0);

                                        let cancel_btn = egui::Button::new(
                                            RichText::new("Cancel").color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_gray(50))
                                        .stroke(Stroke::NONE);
                                        if ui.add(cancel_btn).clicked() {
                                            app.confirm_clear_cache = None;
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(10.0);
                } else {
                    app.confirm_clear_cache = None;
                }
            }

            let mut to_confirm: Option<usize> = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                let snapshot: Vec<_> = app
                    .dev_caches
                    .iter()
                    .enumerate()
                    .map(|(i, c)| (i, c.name.clone(), c.size_bytes))
                    .collect();

                for (idx, name, size) in &snapshot {
                    let idx = *idx;
                    Frame::new()
                        .fill(Color32::from_rgb(24, 24, 24))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(name).color(Color32::WHITE));

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let clear_btn = egui::Button::new(
                                            RichText::new("Clear")
                                                .color(Color32::WHITE)
                                                .size(12.0),
                                        )
                                        .fill(Color32::from_rgb(220, 38, 38))
                                        .stroke(Stroke::NONE);

                                        if ui.add(clear_btn).clicked() {
                                            to_confirm = Some(idx);
                                        }

                                        ui.add_space(8.0);

                                        ui.label(
                                            RichText::new(super::format_bytes(*size))
                                                .color(Color32::from_gray(180)),
                                        );
                                    },
                                );
                            });
                        });
                    ui.add_space(4.0);
                }
            });

            if let Some(idx) = to_confirm {
                app.confirm_clear_cache = Some(idx);
            }
        });
}

use crate::app::SweepApp;
use eframe::egui::{self, Color32, Frame, RichText, ScrollArea, Stroke, Ui};

pub fn draw_launch_agents(ui: &mut Ui, app: &mut SweepApp) {
    Frame::new()
        .inner_margin(egui::Margin::symmetric(24, 24))
        .show(ui, |ui| {
            ui.label(RichText::new("Launch Agents").size(22.0).strong().color(Color32::WHITE));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Background services from apps that are no longer installed.")
                    .color(Color32::from_gray(150)),
            );
            ui.add_space(16.0);

            if app.scanning && app.launch_agents.is_empty() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("  Scanning…").color(Color32::from_gray(150)));
                });
                return;
            }

            if app.launch_agents.is_empty() {
                ui.label(
                    RichText::new("✓ No orphaned launch agents found")
                        .color(Color32::from_rgb(74, 222, 128)),
                );
                return;
            }

            let total_bytes: u64 = app.launch_agents.iter().map(|a| a.size_bytes).sum();
            ui.label(
                RichText::new(format!(
                    "{} agents — {}",
                    app.launch_agents.len(),
                    super::format_bytes(total_bytes)
                ))
                .color(Color32::from_gray(180)),
            );
            ui.add_space(10.0);

            // Inline confirmation banner
            if let Some(pending_idx) = app.confirm_delete_agent {
                if let Some(agent) = app.launch_agents.get(pending_idx) {
                    let name = agent.name.clone();
                    let size = agent.size_bytes;

                    Frame::new()
                        .fill(Color32::from_rgb(30, 20, 20))
                        .corner_radius(6.0)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                        .inner_margin(egui::Margin::symmetric(12, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "Delete \"{}\" · {}?",
                                        name,
                                        super::format_bytes(size)
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
                                            if let Some(item) = app.launch_agents.get(pending_idx) {
                                                let _ = debris_core::delete_path(&item.path);
                                            }
                                            app.launch_agents.remove(pending_idx);
                                            app.confirm_delete_agent = None;
                                        }

                                        ui.add_space(8.0);

                                        let cancel_btn = egui::Button::new(
                                            RichText::new("Cancel").color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_gray(50))
                                        .stroke(Stroke::NONE);
                                        if ui.add(cancel_btn).clicked() {
                                            app.confirm_delete_agent = None;
                                        }
                                    },
                                );
                            });
                        });
                    ui.add_space(10.0);
                } else {
                    app.confirm_delete_agent = None;
                }
            }

            let mut to_confirm: Option<usize> = None;

            ScrollArea::vertical().show(ui, |ui| {
                let snapshot: Vec<_> = app
                    .launch_agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| (i, a.name.clone(), a.size_bytes))
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
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let btn = egui::Button::new(
                                        RichText::new("Delete").color(Color32::WHITE).size(12.0),
                                    )
                                    .fill(Color32::from_rgb(239, 68, 68))
                                    .stroke(Stroke::NONE)
                                    .min_size(egui::vec2(70.0, 24.0));
                                    if ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        to_confirm = Some(idx);
                                    }
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(super::format_bytes(*size))
                                            .color(Color32::from_gray(180)),
                                    );
                                    ui.add_space(8.0);
                                    Frame::new()
                                        .fill(Color32::from_rgb(40, 30, 10))
                                        .corner_radius(4.0)
                                        .inner_margin(egui::Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("LaunchAgent")
                                                    .color(Color32::from_rgb(234, 179, 8))
                                                    .size(11.0),
                                            );
                                        });
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            });

            if let Some(idx) = to_confirm {
                app.confirm_delete_agent = Some(idx);
            }
        });
}

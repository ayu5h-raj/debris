use crate::app::{Section, SweepApp};
use eframe::egui::{self, Color32, Frame, RichText, Sense, Stroke, Ui, Vec2};

pub fn draw_sidebar(ui: &mut Ui, app: &mut SweepApp) {
    Frame::new()
        .fill(Color32::from_rgb(18, 18, 18))
        .inner_margin(egui::Margin::symmetric(12, 16))
        .show(ui, |ui| {
            ui.set_min_width(180.0);

            // Title
            ui.label(
                RichText::new("◉ Debris")
                    .color(Color32::WHITE)
                    .size(18.0)
                    .strong(),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Nav items
            let orphan_count = app.orphans.len();
            let cache_count = app.dev_caches.len();

            nav_item(ui, app, Section::Overview, "Overview", 0);
            nav_item(ui, app, Section::Orphaned, "Orphaned", orphan_count);
            nav_item(ui, app, Section::DevCaches, "Dev Caches", cache_count);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(16.0);

                // Scan button
                let button_color = if app.scanning {
                    Color32::from_rgb(55, 65, 81)
                } else {
                    Color32::from_rgb(59, 130, 246)
                };

                let scan_label = if app.scanning { "Scanning…" } else { "Scan" };

                let btn = egui::Button::new(RichText::new(scan_label).color(Color32::WHITE).strong())
                    .fill(button_color)
                    .stroke(Stroke::NONE)
                    .min_size(Vec2::new(156.0, 32.0));

                let response = ui.add_enabled(!app.scanning, btn);
                if response.clicked() {
                    app.start_scan();
                }
            });
        });
}

fn nav_item(ui: &mut Ui, app: &mut SweepApp, section: Section, label: &str, badge: usize) {
    let is_selected = app.section == section;

    let bg = if is_selected {
        Color32::from_rgb(35, 35, 55)
    } else {
        Color32::TRANSPARENT
    };

    let text_color = if is_selected {
        Color32::WHITE
    } else {
        Color32::from_gray(180)
    };

    let frame = Frame::new()
        .fill(bg)
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 6));

    let response = frame
        .show(ui, |ui| {
            ui.set_min_width(156.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).color(text_color));
                if badge > 0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let badge_text = if badge > 99 {
                            "99+".to_string()
                        } else {
                            badge.to_string()
                        };
                        ui.label(
                            RichText::new(badge_text)
                                .color(Color32::WHITE)
                                .size(10.0)
                                .background_color(Color32::from_rgb(239, 68, 68)),
                        );
                    });
                }
            });
        })
        .response;

    let interact = ui.interact(response.rect, ui.next_auto_id(), Sense::click());
    if interact.clicked() {
        app.section = section;
    }
}

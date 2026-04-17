use crate::app::TuiApp;
use super::format_bytes;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn render_overview(f: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let Some(disk) = &app.disk_info else {
        f.render_widget(
            Paragraph::new("No disk info available.")
                .block(Block::default().borders(Borders::ALL).title(" Storage Overview ")),
            area,
        );
        return;
    };

    let pct = if disk.total_bytes > 0 {
        ((disk.used_bytes as f64 / disk.total_bytes as f64 * 100.0) as u16).min(100)
    } else {
        0
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Disk Usage "))
        .gauge_style(Style::default().fg(Color::Blue).bg(Color::DarkGray))
        .percent(pct)
        .label(format!(
            "{}% — {} used of {}",
            pct,
            format_bytes(disk.used_bytes),
            format_bytes(disk.total_bytes),
        ));
    f.render_widget(gauge, chunks[0]);

    let stats = Line::from(vec![
        Span::styled(" Free: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format_bytes(disk.free_bytes), Style::default().fg(Color::White)),
        Span::raw("    "),
        Span::styled("Total: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format_bytes(disk.total_bytes), Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(stats), chunks[1]);

    let orphan_bytes: u64 = app.orphans.iter().map(|o| o.total_size).sum();
    let cache_bytes: u64 = app.dev_caches.iter().map(|c| c.size_bytes).sum();
    let agent_bytes: u64 = app.launch_agents.iter().map(|a| a.size_bytes).sum();

    let categories = [
        ("Orphaned App Data", orphan_bytes, Color::Red),
        ("Dev Caches", cache_bytes, Color::Yellow),
        ("Launch Agents", agent_bytes, Color::Magenta),
    ];

    for (i, (label, bytes, color)) in categories.iter().enumerate() {
        let pct_cat = if disk.total_bytes > 0 {
            (*bytes as f64 / disk.total_bytes as f64 * 100.0) as u16
        } else {
            0
        };
        let row = Line::from(vec![
            Span::styled(format!(" {:<22}", label), Style::default().fg(Color::White)),
            Span::styled(
                format!("{:>10}", format_bytes(*bytes)),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  ({}%)", pct_cat), Style::default().fg(Color::DarkGray)),
        ]);
        f.render_widget(Paragraph::new(row), chunks[2 + i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_overview_renders_without_disk_info() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.disk_info = None;
        terminal.draw(|f| render_overview(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("No disk info"));
    }

    #[test]
    fn test_overview_shows_disk_gauge() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.disk_info = Some(debris_core::DiskInfo {
            total_bytes: 1_000_000_000,
            used_bytes: 750_000_000,
            free_bytes: 250_000_000,
        });
        terminal.draw(|f| render_overview(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("75"));
    }
}

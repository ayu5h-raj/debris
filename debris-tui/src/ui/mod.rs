mod overview;
mod orphaned;
mod dev_caches;
mod launch_agents;

use crate::app::{Tab, TuiApp};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render(f: &mut Frame, app: &mut TuiApp) {
    let area = f.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    let tab_index = match app.tab {
        Tab::Overview => 0,
        Tab::Orphaned => 1,
        Tab::DevCaches => 2,
        Tab::LaunchAgents => 3,
    };

    let tabs = Tabs::new(vec![
        "Overview".to_string(),
        format!("Orphaned ({})", app.orphans.len()),
        format!("Dev Caches ({})", app.dev_caches.len()),
        format!("Launch Agents ({})", app.launch_agents.len()),
    ])
    .select(tab_index)
    .block(Block::default().borders(Borders::ALL).title(" ◉ Debris "))
    .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .style(Style::default().fg(Color::White));
    f.render_widget(tabs, chunks[0]);

    match app.tab {
        Tab::Overview => overview::render_overview(f, chunks[1], app),
        Tab::Orphaned => orphaned::render_orphaned(f, chunks[1], app),
        Tab::DevCaches => dev_caches::render_dev_caches(f, chunks[1], app),
        Tab::LaunchAgents => launch_agents::render_launch_agents(f, chunks[1], app),
    }

    let scanning_hint = if app.scanning { "  scanning…" } else { "  r rescan" };
    let help = Line::from(vec![
        Span::styled(" ↑↓/jk ", Style::default().fg(Color::Yellow)),
        Span::raw("navigate"),
        Span::styled("  space ", Style::default().fg(Color::Yellow)),
        Span::raw("select"),
        Span::styled("  d ", Style::default().fg(Color::Yellow)),
        Span::raw("delete"),
        Span::styled("  tab ", Style::default().fg(Color::Yellow)),
        Span::raw("switch"),
        Span::styled(scanning_hint, Style::default().fg(Color::DarkGray)),
        Span::styled("  q ", Style::default().fg(Color::Yellow)),
        Span::raw("quit"),
    ]);
    f.render_widget(Paragraph::new(help), chunks[2]);
}

pub fn format_bytes(bytes: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_gb() { assert_eq!(format_bytes(2_500_000_000), "2.5 GB"); }
    #[test]
    fn test_format_bytes_mb() { assert_eq!(format_bytes(1_500_000), "1.5 MB"); }
    #[test]
    fn test_format_bytes_kb() { assert_eq!(format_bytes(2_048), "2.0 KB"); }
    #[test]
    fn test_format_bytes_b() { assert_eq!(format_bytes(500), "500 B"); }

    #[test]
    fn test_render_does_not_panic() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.scanning = false;
        terminal.draw(|f| render(f, &mut app)).unwrap();
    }
}

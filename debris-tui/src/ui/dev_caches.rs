use crate::app::{ConfirmAction, TuiApp};
use super::format_bytes;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_dev_caches(f: &mut Frame, area: Rect, app: &TuiApp) {
    if app.dev_caches.is_empty() {
        f.render_widget(
            Paragraph::new(if app.scanning { "Scanning…" } else { "✓ No dev caches found." })
                .block(Block::default().borders(Borders::ALL).title(" Dev Caches ")),
            area,
        );
        return;
    }

    let total: u64 = app.dev_caches.iter().map(|c| c.size_bytes).sum();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Dev Caches — {} caches  {}  d clear ", app.dev_caches.len(), format_bytes(total)));

    let mut sorted: Vec<(usize, &debris_core::DevCacheItem)> = app.dev_caches.iter().enumerate().collect();
    sorted.sort_unstable_by(|a, b| b.1.size_bytes.cmp(&a.1.size_bytes));

    let items: Vec<ListItem> = sorted
        .iter()
        .map(|(_, item)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("  {:<30}", item.name)),
                Span::styled(
                    format!("{:>10}", format_bytes(item.size_bytes)),
                    Style::default().fg(Color::Yellow),
                ),
            ]))
        })
        .collect();

    let display_cursor = sorted
        .iter()
        .position(|(orig, _)| *orig == app.cache_cursor)
        .unwrap_or(0);

    let mut state = ListState::default();
    state.select(Some(display_cursor.min(sorted.len().saturating_sub(1))));

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );

    if let Some(ConfirmAction::ClearCache(idx)) = &app.confirm {
        if let Some((_, item)) = sorted.get(*idx) {
            let msg = format!("Clear \"{}\" ({})? [y/n]", item.name, format_bytes(item.size_bytes));
            render_confirm_popup(f, area, &msg);
        }
    }
}

fn render_confirm_popup(f: &mut Frame, area: Rect, message: &str) {
    let popup_area = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(5),
        Constraint::Percentage(40),
    ])
    .split(area)[1];
    let popup_area = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(60),
        Constraint::Percentage(20),
    ])
    .split(popup_area)[1];
    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title(" Confirm ").style(Style::default().fg(Color::Red)))
            .alignment(Alignment::Center),
        popup_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_dev_caches_empty() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.scanning = false;
        terminal.draw(|f| render_dev_caches(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("No dev caches"));
    }

    #[test]
    fn test_dev_caches_shows_name() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.scanning = false;
        app.dev_caches = vec![debris_core::DevCacheItem {
            name: "npm".into(),
            path: std::path::PathBuf::from("/tmp/npm"),
            size_bytes: 500_000_000,
        }];
        terminal.draw(|f| render_dev_caches(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("npm"));
    }
}

use crate::app::{ConfirmAction, TuiApp};
use super::format_bytes;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_orphaned(f: &mut Frame, area: Rect, app: &TuiApp) {
    if app.scanning && app.orphans.is_empty() {
        f.render_widget(
            Paragraph::new("Scanning…")
                .block(Block::default().borders(Borders::ALL).title(" Orphaned App Data ")),
            area,
        );
        return;
    }

    if app.orphans.is_empty() {
        f.render_widget(
            Paragraph::new("✓ No orphaned data found.")
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title(" Orphaned App Data ")),
            area,
        );
        return;
    }

    let total: u64 = app.orphans.iter().map(|o| o.total_size).sum();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " Orphaned App Data — {} items  {}  space select  d delete ",
            app.orphans.len(),
            format_bytes(total),
        ));

    let mut sorted: Vec<(usize, &debris_core::OrphanItem)> = app.orphans.iter().enumerate().collect();
    sorted.sort_unstable_by(|a, b| b.1.total_size.cmp(&a.1.total_size));

    let items: Vec<ListItem> = sorted
        .iter()
        .map(|(orig_idx, item)| {
            let selected = app.selected.contains(orig_idx);
            let prefix = if selected { "✓ " } else { "  " };
            let source_tag = match item.source {
                debris_core::OrphanSource::KnownDb => "[DB]",
                debris_core::OrphanSource::Heuristic => "[~]",
                debris_core::OrphanSource::Containers => "[C]",
            };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Green)),
                Span::raw(format!("{:<45}", &item.name)),
                Span::styled(
                    format!("{:>10}", format_bytes(item.total_size)),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("  {}", source_tag),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            let style = if selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let display_cursor = sorted
        .iter()
        .position(|(orig, _)| *orig == app.orphan_cursor)
        .unwrap_or(0);

    let mut state = ListState::default();
    state.select(Some(display_cursor));

    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ "),
        area,
        &mut state,
    );

    if let Some(ConfirmAction::DeleteOrphans) = &app.confirm {
        render_confirm_popup(f, area, &format!("Delete {} item(s)? [y/n]", app.selected.len()));
    }
}

fn render_confirm_popup(f: &mut Frame, area: Rect, message: &str) {
    let popup_area = centered_rect(50, 20, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm ")
                    .style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center),
        popup_area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_orphaned_empty_state() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.scanning = false;
        app.orphans = vec![];
        terminal.draw(|f| render_orphaned(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("No orphaned"));
    }

    #[test]
    fn test_orphaned_shows_item_name() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = TuiApp::new();
        app.scanning = false;
        app.orphans = vec![debris_core::OrphanItem {
            name: "com.example.gone".into(),
            paths: vec![],
            total_size: 50_000_000,
            source: debris_core::OrphanSource::Heuristic,
        }];
        terminal.draw(|f| render_orphaned(f, f.area(), &app)).unwrap();
        let content: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("com.example.gone"));
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{Tab, TuiApp};

pub fn handle_key(app: &mut TuiApp, key: KeyEvent) -> bool {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return false;
    }

    // Confirmation dialog captures all input
    if app.confirm.is_some() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => app.confirm_action(),
            KeyCode::Char('n') | KeyCode::Esc => app.cancel_confirm(),
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
        KeyCode::Tab => app.next_tab(),
        KeyCode::Char('1') => app.tab = Tab::Overview,
        KeyCode::Char('2') => app.tab = Tab::Orphaned,
        KeyCode::Char('3') => app.tab = Tab::DevCaches,
        KeyCode::Char('4') => app.tab = Tab::LaunchAgents,
        KeyCode::Down | KeyCode::Char('j') => app.cursor_down(),
        KeyCode::Up | KeyCode::Char('k') => app.cursor_up(),
        KeyCode::Char(' ') => app.toggle_select(),
        KeyCode::Char('d') | KeyCode::Delete => app.request_delete(),
        KeyCode::Char('r') => app.start_scan(),
        _ => {}
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ConfirmAction;
    use crossterm::event::KeyEventKind;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    fn make_app() -> TuiApp {
        let mut app = TuiApp::new();
        app.scanning = false;
        app.scan_rx = None;
        app
    }

    #[test]
    fn test_q_quits() {
        let mut app = make_app();
        assert!(handle_key(&mut app, key(KeyCode::Char('q'))));
    }

    #[test]
    fn test_tab_switches_section() {
        let mut app = make_app();
        app.tab = Tab::Overview;
        handle_key(&mut app, key(KeyCode::Tab));
        assert_eq!(app.tab, Tab::Orphaned);
    }

    #[test]
    fn test_number_keys_jump_to_tab() {
        let mut app = make_app();
        handle_key(&mut app, key(KeyCode::Char('3')));
        assert_eq!(app.tab, Tab::DevCaches);
        handle_key(&mut app, key(KeyCode::Char('1')));
        assert_eq!(app.tab, Tab::Overview);
    }

    #[test]
    fn test_confirm_y_executes_action() {
        let mut app = make_app();
        app.tab = Tab::DevCaches;
        app.dev_caches = vec![debris_core::DevCacheItem {
            name: "npm".into(),
            path: std::path::PathBuf::from("/nonexistent/path"),
            size_bytes: 100,
        }];
        app.confirm = Some(ConfirmAction::ClearCache(0));
        handle_key(&mut app, key(KeyCode::Char('y')));
        assert!(app.confirm.is_none());
    }

    #[test]
    fn test_confirm_n_cancels() {
        let mut app = make_app();
        app.confirm = Some(ConfirmAction::DeleteOrphans);
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert!(app.confirm.is_none());
    }
}

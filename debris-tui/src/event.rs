use crossterm::event::KeyEvent;
use crate::app::TuiApp;
pub fn handle_key(_app: &mut TuiApp, _key: KeyEvent) -> bool { false }

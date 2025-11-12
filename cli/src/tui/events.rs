use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Tab,
    Esc,
    Char(char),
    Tick,
    MouseScroll { up: bool, column: u16, row: u16 },
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> anyhow::Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) => return Ok(self.handle_key_event(key)),
                Event::Mouse(mouse) => return Ok(self.handle_mouse_event(mouse)),
                _ => {}
            }
        }
        Ok(AppEvent::Tick)
    }

    fn handle_key_event(&self, key: KeyEvent) -> AppEvent {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => AppEvent::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => AppEvent::Quit,
            KeyCode::Up => AppEvent::Up,
            KeyCode::Down => AppEvent::Down,
            KeyCode::Left => AppEvent::Left,
            KeyCode::Right => AppEvent::Right,
            KeyCode::Enter => AppEvent::Enter,
            KeyCode::Tab => AppEvent::Tab,
            KeyCode::Esc => AppEvent::Esc,
            KeyCode::Char(c) => AppEvent::Char(c),
            _ => AppEvent::Tick,
        }
    }

    fn handle_mouse_event(&self, mouse: MouseEvent) -> AppEvent {
        match mouse.kind {
            MouseEventKind::ScrollUp => AppEvent::MouseScroll {
                up: true,
                column: mouse.column,
                row: mouse.row,
            },
            MouseEventKind::ScrollDown => AppEvent::MouseScroll {
                up: false,
                column: mouse.column,
                row: mouse.row,
            },
            _ => AppEvent::Tick,
        }
    }
}


use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum MenuAction {
    None,
    QuickStart,
    SelectMode,
    Exit,
}

#[derive(Debug)]
pub struct MenuWidget {
    selected_index: usize,
    options: Vec<String>,
}

impl MenuWidget {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            options: vec![
                "Quick Start".to_string(),
                "Select Mode".to_string(),
                "Exit".to_string(),
            ],
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> MenuAction {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                MenuAction::None
            }
            KeyCode::Down => {
                if self.selected_index < self.options.len() - 1 {
                    self.selected_index += 1;
                }
                MenuAction::None
            }
            KeyCode::Enter => {
                match self.selected_index {
                    0 => MenuAction::QuickStart,
                    1 => MenuAction::SelectMode,
                    2 => MenuAction::Exit,
                    _ => MenuAction::None,
                }
            }
            _ => MenuAction::None,
        }
    }

    pub fn reset(&mut self) {
        self.selected_index = 0;
    }
}

impl Widget for &MenuWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = self.options.len();
        let start_y = area.y + area.height.saturating_sub(1) / 2 - total_lines as u16 / 2;

        for (i, option) in self.options.iter().enumerate() {
            let y = start_y + i as u16;
            if y < area.y || y >= area.y + area.height {
                continue;
            }

            let line = if i == self.selected_index {
                Line::from(format!("> {}", option)).style(Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {}", option)).style(Style::default().fg(Color::White))
            };

            let option_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            Paragraph::new(line)
                .alignment(Alignment::Center)
                .render(option_area, buf);
        }
    }
}
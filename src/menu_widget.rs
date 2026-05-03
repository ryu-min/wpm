use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};
use crate::menu_widget::MenuAction::Exit;

#[derive(Debug, PartialEq, Clone)]
pub enum MenuAction {
    QuickStart,
    SelectMode,
    Exit,
}

#[derive(Debug, Clone)]
struct MenuItem {
    text: String,
    action: MenuAction,
}

#[derive(Debug)]
pub struct MenuWidget {
    selected_index: usize,
    items: Vec<MenuItem>,
}

impl MenuWidget {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            items: vec![
                MenuItem { text: "Quick Start".to_string(), action: MenuAction::QuickStart },
                MenuItem { text: "Select Mode".to_string(), action: MenuAction::SelectMode },
                MenuItem { text: "Exit".to_string(), action: MenuAction::Exit },
            ],
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<MenuAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
                None
            }
            KeyCode::Enter => {
                Some(self.items[self.selected_index].action.clone())
            }
            KeyCode::Esc => {
                Some(Exit)
            }
            _ => None,
        }
    }

    pub fn reset(&mut self) {
        self.selected_index = 0;
    }
}

impl Widget for &MenuWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = self.items.len();
        let start_y = area.y + area.height.saturating_sub(1) / 2 - total_lines as u16 / 2;

        for (i, item) in self.items.iter().enumerate() {
            let y = start_y + i as u16;
            if y < area.y || y >= area.y + area.height {
                continue;
            }

            let line = if i == self.selected_index {
                Line::from(format!("> {}", item.text)).style(Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {}", item.text)).style(Style::default().fg(Color::White))
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
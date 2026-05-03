use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum ResultAction {
    Restart,
    Menu,
}

#[derive(Debug, Clone)]
struct ResultItem {
    text: String,
    action: ResultAction,
}

#[derive(Debug)]
pub struct ResultWidget {
    wpm: f64,
    accuracy: f64,
    time: f64,
    selected_index: usize,
    items: Vec<ResultItem>,
}

impl ResultWidget {
    pub fn new() -> Self {
        Self {
            wpm: 0.0,
            accuracy: 0.0,
            time: 0.0,
            selected_index: 0,
            items: vec![
                ResultItem { text: "Restart".to_string(), action: ResultAction::Restart },
                ResultItem { text: "Menu".to_string(), action: ResultAction::Menu },
            ],
        }
    }

    pub fn update(&mut self, wpm: f64, accuracy: f64, time: f64) {
        self.wpm = wpm;
        self.accuracy = accuracy;
        self.time = time;
    }
    
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ResultAction> {
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
            KeyCode::Esc => Some(ResultAction::Menu),
            _ => None,
        }
    }

    pub fn reset(&mut self) {
        self.selected_index = 0;
    }
}

impl Widget for &ResultWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 3 + self.items.len();
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let wpm_line = Line::from(format!("WPM: {}", self.wpm as u32))
            .style(Style::default().fg(Color::Cyan));
        let acc_line = Line::from(format!("Accuracy: {:.0}%", self.accuracy * 100.0))
            .style(Style::default().fg(Color::Green));
        let time_line = Line::from(format!("Time: {:.1}s", self.time))
            .style(Style::default().fg(Color::Yellow));

        Paragraph::new(wpm_line).alignment(ratatui::layout::Alignment::Center).render(
            Rect { x: area.x, y: start_y, width: area.width, height: 1 }, buf
        );
        Paragraph::new(acc_line).alignment(ratatui::layout::Alignment::Center).render(
            Rect { x: area.x, y: start_y + 1, width: area.width, height: 1 }, buf
        );
        Paragraph::new(time_line).alignment(ratatui::layout::Alignment::Center).render(
            Rect { x: area.x, y: start_y + 2, width: area.width, height: 1 }, buf
        );

        for (i, item) in self.items.iter().enumerate() {
            let line = if i == self.selected_index {
                Line::from(format!("> {}", item.text)).style(Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {}", item.text)).style(Style::default().fg(Color::White))
            };

            Paragraph::new(line).alignment(ratatui::layout::Alignment::Center).render(
                Rect { x: area.x, y: start_y + 4 + i as u16, width: area.width, height: 1 }, buf
            );
        }
    }
}
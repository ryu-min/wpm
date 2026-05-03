use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum ResultAction {
    None,
    Restart,
    Menu,
}

#[derive(Debug)]
pub struct ResultWidget {
    wpm: f64,
    accuracy: f64,
    time: f64,
    selected_index: usize,
}

impl ResultWidget {
    pub fn new() -> Self {
        Self {
            wpm: 0.0,
            accuracy: 0.0,
            time: 0.0,
            selected_index: 0,
        }
    }

    pub fn update(&mut self, wpm: f64, accuracy: f64, time: f64) {
        self.wpm = wpm;
        self.accuracy = accuracy;
        self.time = time;
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> ResultAction {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                ResultAction::None
            }
            KeyCode::Down => {
                if self.selected_index < 1 {
                    self.selected_index += 1;
                }
                ResultAction::None
            }
            KeyCode::Enter => {
                match self.selected_index {
                    0 => ResultAction::Restart,
                    1 => ResultAction::Menu,
                    _ => ResultAction::None,
                }
            }
            _ => ResultAction::None,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < 1 {
            self.selected_index += 1;
        }
    }

    pub fn reset(&mut self) {
        self.selected_index = 0;
    }
}

impl Widget for &ResultWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let options = vec!["Restart", "Menu"];
        let total_lines = 5;
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

        for (i, option) in options.iter().enumerate() {
            let line = if i == self.selected_index {
                Line::from(format!("> {}", option)).style(Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {}", option)).style(Style::default().fg(Color::White))
            };

            Paragraph::new(line).alignment(ratatui::layout::Alignment::Center).render(
                Rect { x: area.x, y: start_y + 4 + i as u16, width: area.width, height: 1 }, buf
            );
        }
    }
}
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum SettingsAction {
    None,
    Save,
    Exit,
}

#[derive(Debug)]
pub struct SettingsWidget {
    active_row: usize,
    mode_index: usize,
    wordset_index: usize,
    translation_set_index: usize,
    time_index: usize,
    mode_options: Vec<String>,
    wordset_options: Vec<String>,
    translation_set_options: Vec<String>,
    time_options: Vec<u32>,
}

impl SettingsWidget {
    pub fn new(
        wordset_names: Vec<String>,
        translation_set_names: Vec<String>,
        current_mode: &str,
        current_wordset: &str,
        current_translation_set: &str,
        current_time: u32,
    ) -> Self {
        let times: Vec<u32> = vec![15, 30, 45, 60, 90, 120, 180, 300];
        let mode_options = vec!["typing".to_string(), "translation".to_string()];
        let mode_index = mode_options
            .iter()
            .position(|m| m == current_mode)
            .unwrap_or(0);
        let wordset_index = wordset_names.iter().position(|n| n == current_wordset).unwrap_or(0);
        let translation_set_index = translation_set_names
            .iter()
            .position(|n| n == current_translation_set)
            .unwrap_or(0);
        let time_index = times.iter().position(|&t| t == current_time).unwrap_or(0);
        
        Self {
            active_row: 0,
            mode_index,
            wordset_index,
            translation_set_index,
            time_index,
            mode_options,
            wordset_options: wordset_names,
            translation_set_options: translation_set_names,
            time_options: times,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                self.active_row = if self.active_row > 0 { self.active_row - 1 } else { 3 };
                None
            }
            KeyCode::Down => {
                self.active_row = if self.active_row < 3 { self.active_row + 1 } else { 0 };
                None
            }
            KeyCode::Left => {
                if self.active_row == 0 {
                    if self.mode_index > 0 {
                        self.mode_index -= 1;
                    }
                } else if self.active_row == 1 {
                    if self.wordset_index > 0 {
                        self.wordset_index -= 1;
                    }
                } else if self.active_row == 2 {
                    if self.translation_set_index > 0 {
                        self.translation_set_index -= 1;
                    }
                } else {
                    if self.time_index > 0 {
                        self.time_index -= 1;
                    }
                }
                None
            }
            KeyCode::Right => {
                if self.active_row == 0 {
                    if self.mode_index < self.mode_options.len() - 1 {
                        self.mode_index += 1;
                    }
                } else if self.active_row == 1 {
                    if self.wordset_index < self.wordset_options.len() - 1 {
                        self.wordset_index += 1;
                    }
                } else if self.active_row == 2 {
                    if self.translation_set_index < self.translation_set_options.len() - 1 {
                        self.translation_set_index += 1;
                    }
                } else {
                    if self.time_index < self.time_options.len() - 1 {
                        self.time_index += 1;
                    }
                }
                None
            }
            KeyCode::Enter => Some(SettingsAction::Save),
            KeyCode::Esc => Some(SettingsAction::Exit),
            _ => None,
        }
    }

    pub fn current_wordset(&self) -> &str {
        &self.wordset_options[self.wordset_index]
    }

    pub fn current_mode(&self) -> &str {
        &self.mode_options[self.mode_index]
    }

    pub fn current_translation_set(&self) -> &str {
        if self.translation_set_options.is_empty() {
            "ru_en_a1"
        } else {
            &self.translation_set_options[self.translation_set_index]
        }
    }

    pub fn current_time(&self) -> u32 {
        self.time_options[self.time_index]
    }
}

impl Widget for &SettingsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 15;
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let title = Paragraph::new(Line::from("Settings").style(Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center);
        title.render(Rect { x: area.x, y: start_y, width: area.width, height: 1 }, buf);

        let quick_start_label = Paragraph::new(Line::from("Quick Start:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        quick_start_label.render(Rect { x: area.x, y: start_y + 2, width: area.width, height: 1 }, buf);

        let mode_label = Paragraph::new(Line::from("Mode:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        mode_label.render(Rect { x: area.x, y: start_y + 3, width: area.width, height: 1 }, buf);

        let mode_style = if self.active_row == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let mode_value = Line::from(format!("< {} >", self.mode_options[self.mode_index])).style(mode_style);
        Paragraph::new(mode_value).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 4, width: area.width, height: 1 }, buf
        );

        let is_typing_mode = self.mode_options[self.mode_index] == "typing";
        let wordset_style = if self.active_row == 1 {
            Style::default().fg(Color::Yellow)
        } else if is_typing_mode {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let wordset_value = Line::from(format!("< {} >", self.wordset_options[self.wordset_index])).style(wordset_style);
        Paragraph::new(wordset_value).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 6, width: area.width, height: 1 }, buf
        );

        let trs_label = Paragraph::new(Line::from("Translation set:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        trs_label.render(Rect { x: area.x, y: start_y + 7, width: area.width, height: 1 }, buf);

        let is_translation_mode = !is_typing_mode;
        let trs_style = if self.active_row == 2 {
            Style::default().fg(Color::Yellow)
        } else if is_translation_mode {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let trs_value = if self.translation_set_options.is_empty() {
            "< empty >".to_string()
        } else {
            format!("< {} >", self.translation_set_options[self.translation_set_index])
        };
        Paragraph::new(Line::from(trs_value).style(trs_style)).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 8, width: area.width, height: 1 }, buf
        );

        let time_label = Paragraph::new(Line::from("Time:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        time_label.render(Rect { x: area.x, y: start_y + 10, width: area.width, height: 1 }, buf);

        let seconds = self.time_options[self.time_index];
        let time_str = if seconds >= 60 {
            let min = seconds / 60;
            let sec = seconds % 60;
            if sec == 0 {
                format!("{} min", min)
            } else {
                format!("{} min {} sec", min, sec)
            }
        } else {
            format!("{} sec", seconds)
        };
        let time_style = if self.active_row == 3 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        let time_value = Line::from(format!("< {} >", time_str)).style(time_style);
        Paragraph::new(time_value).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 11, width: area.width, height: 1 }, buf
        );

        let wordset_label = Paragraph::new(Line::from("Wordset:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        wordset_label.render(Rect { x: area.x, y: start_y + 5, width: area.width, height: 1 }, buf);

        let hint = Paragraph::new(
            Line::from("↑ ↓ : select row  ← → : change value  Enter : save  Esc : back").style(Style::default().fg(Color::DarkGray))
        ).alignment(Alignment::Center);
        hint.render(Rect { x: area.x, y: start_y + 14, width: area.width, height: 1 }, buf);
    }
}

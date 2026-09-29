use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum ModeSelectAction {
    Start,
    Exit,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TestMode {
    Typing,
    Translation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveRow {
    Mode,
    Primary,
    Time,
}

pub struct ModeSelectWidget {
    active_row: ActiveRow,
    mode_index: usize,
    wordset_index: usize,
    translation_set_index: usize,
    time_index: usize,
    wordset_options: Vec<String>,
    translation_set_options: Vec<String>,
    time_options: Vec<u32>,
}

impl std::fmt::Debug for ModeSelectWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModeSelectWidget")
            .field("mode_index", &self.mode_index)
            .field("wordset_index", &self.wordset_index)
            .field("translation_set_index", &self.translation_set_index)
            .field("time_index", &self.time_index)
            .finish()
    }
}

impl ModeSelectWidget {
    pub fn new(wordset_names: Vec<String>, translation_set_names: Vec<String>) -> Self {
        let times: Vec<u32> = (15..=300).step_by(15).collect();
        Self {
            active_row: ActiveRow::Mode,
            mode_index: 0,
            wordset_index: 0,
            translation_set_index: 0,
            time_index: 0,
            wordset_options: wordset_names,
            translation_set_options: translation_set_names,
            time_options: times,
        }
    }

    pub fn wordset_index(&self) -> usize {
        self.wordset_index
    }

    pub fn time_index(&self) -> usize {
        self.time_index
    }

    pub fn selected_time(&self) -> u32 {
        self.time_options[self.time_index]
    }

    pub fn selected_wordset(&self) -> &str {
        &self.wordset_options[self.wordset_index]
    }

    pub fn selected_mode(&self) -> TestMode {
        if self.mode_index == 0 {
            TestMode::Typing
        } else {
            TestMode::Translation
        }
    }

    pub fn selected_translation_set(&self) -> &str {
        if self.translation_set_options.is_empty() {
            "ru_en_a1"
        } else {
            &self.translation_set_options[self.translation_set_index]
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<ModeSelectAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                self.move_up();
                None
            }
            KeyCode::Down => {
                self.move_down();
                None
            }
            KeyCode::Left => {
                self.decrement_value();
                None
            }
            KeyCode::Right => {
                self.increment_value();
                None
            }
            KeyCode::Enter => Some(ModeSelectAction::Start),
            KeyCode::Esc => Some(ModeSelectAction::Exit),
            _ => None,
        }
    }

    fn move_up(&mut self) {
        self.active_row = match self.active_row {
            ActiveRow::Mode => ActiveRow::Time,
            ActiveRow::Primary => ActiveRow::Mode,
            ActiveRow::Time => ActiveRow::Primary,
        };
    }

    fn move_down(&mut self) {
        self.active_row = match self.active_row {
            ActiveRow::Mode => ActiveRow::Primary,
            ActiveRow::Primary => ActiveRow::Time,
            ActiveRow::Time => ActiveRow::Mode,
        };
    }

    fn decrement_value(&mut self) {
        match (self.selected_mode(), self.active_row) {
            (_, ActiveRow::Mode) => {
                if self.mode_index > 0 {
                    self.mode_index -= 1;
                }
            }
            (TestMode::Typing, ActiveRow::Primary) => {
                if self.wordset_index > 0 {
                    self.wordset_index -= 1;
                }
            }
            (TestMode::Translation, ActiveRow::Primary) => {
                if self.translation_set_index > 0 {
                    self.translation_set_index -= 1;
                }
            }
            (_, ActiveRow::Time) => {
                if self.time_index > 0 {
                    self.time_index -= 1;
                }
            }
        }
    }

    fn increment_value(&mut self) {
        match (self.selected_mode(), self.active_row) {
            (_, ActiveRow::Mode) => {
                if self.mode_index < 1 {
                    self.mode_index += 1;
                }
            }
            (TestMode::Typing, ActiveRow::Primary) => {
                if self.wordset_index + 1 < self.wordset_options.len() {
                    self.wordset_index += 1;
                }
            }
            (TestMode::Translation, ActiveRow::Primary) => {
                if self.translation_set_index + 1 < self.translation_set_options.len() {
                    self.translation_set_index += 1;
                }
            }
            (_, ActiveRow::Time) => {
                if self.time_index + 1 < self.time_options.len() {
                    self.time_index += 1;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.active_row = ActiveRow::Mode;
        self.mode_index = 0;
        self.wordset_index = 0;
        self.translation_set_index = 0;
        self.time_index = 0;
    }
}

impl Widget for &ModeSelectWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 12;
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let title =
            Paragraph::new(Line::from("Select Mode").style(Style::default().fg(Color::Cyan)))
                .alignment(Alignment::Center);
        title.render(
            Rect {
                x: area.x,
                y: start_y,
                width: area.width,
                height: 1,
            },
            buf,
        );

        let mode_label =
            Paragraph::new(Line::from("Mode:").style(Style::default().fg(Color::White)))
                .alignment(Alignment::Center);
        mode_label.render(
            Rect {
                x: area.x,
                y: start_y + 2,
                width: area.width,
                height: 1,
            },
            buf,
        );

        let mode_name = match self.selected_mode() {
            TestMode::Typing => "Typing",
            TestMode::Translation => "Translation",
        };
        let mode_style = if self.active_row == ActiveRow::Mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        Paragraph::new(Line::from(format!("< {} >", mode_name)).style(mode_style))
            .alignment(Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 3,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        let primary_label_text = match self.selected_mode() {
            TestMode::Typing => "Wordset:",
            TestMode::Translation => "Translation set:",
        };
        let primary_value_text = match self.selected_mode() {
            TestMode::Typing => format!("< {} >", self.wordset_options[self.wordset_index]),
            TestMode::Translation => {
                if self.translation_set_options.is_empty() {
                    "< empty >".to_string()
                } else {
                    format!(
                        "< {} >",
                        self.translation_set_options[self.translation_set_index]
                    )
                }
            }
        };
        Paragraph::new(Line::from(primary_label_text).style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 5,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        let primary_style = match self.active_row {
            ActiveRow::Primary => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::White),
        };
        Paragraph::new(Line::from(primary_value_text).style(primary_style))
            .alignment(Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 6,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        let time_label =
            Paragraph::new(Line::from("Time:").style(Style::default().fg(Color::White)))
                .alignment(Alignment::Center);
        time_label.render(
            Rect {
                x: area.x,
                y: start_y + 8,
                width: area.width,
                height: 1,
            },
            buf,
        );

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
        let time_value_style = match self.active_row {
            ActiveRow::Time => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::White),
        };
        let time_value = Line::from(format!("< {} >", time_str)).style(time_value_style);
        Paragraph::new(time_value)
            .alignment(Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 9,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        let hint = Paragraph::new(
            Line::from("↑ ↓ : select row  ← → : change value  Enter : start  Esc : back")
                .style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);
        hint.render(
            Rect {
                x: area.x,
                y: start_y + 11,
                width: area.width,
                height: 1,
            },
            buf,
        );
    }
}

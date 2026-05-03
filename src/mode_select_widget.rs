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

#[derive(Debug)]
enum ActiveRow {
    Wordset,
    Time,
}

pub struct ModeSelectWidget {
    active_row: ActiveRow,
    wordset_index: usize,
    time_index: usize,
    wordset_options: Vec<String>,
    time_options: Vec<u32>,
}

impl std::fmt::Debug for ModeSelectWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModeSelectWidget")
            .field("wordset_index", &self.wordset_index)
            .field("time_index", &self.time_index)
            .finish()
    }
}

impl ModeSelectWidget {
    pub fn new(wordset_names: Vec<String>) -> Self {
        let times: Vec<u32> = (15..=300).step_by(15).collect();
        Self {
            active_row: ActiveRow::Wordset,
            wordset_index: 0,
            time_index: 0,
            wordset_options: wordset_names,
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
                self.move_left();
                None
            }
            KeyCode::Right => {
                self.move_right();
                None
            }
            KeyCode::Enter => Some(ModeSelectAction::Start),
            KeyCode::Esc => Some(ModeSelectAction::Exit),
            _ => None,
        }
    }

    pub fn move_up(&mut self) {
        match self.active_row {
            ActiveRow::Wordset => {
                self.active_row = ActiveRow::Time;
            }
            ActiveRow::Time => {
                self.active_row = ActiveRow::Wordset;
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.active_row {
            ActiveRow::Wordset => {
                self.active_row = ActiveRow::Time;
            }
            ActiveRow::Time => {
                self.active_row = ActiveRow::Wordset;
            }
        }
    }

    pub fn move_left(&mut self) {
        match self.active_row {
            ActiveRow::Wordset => {
                if self.wordset_index > 0 {
                    self.wordset_index -= 1;
                }
            }
            ActiveRow::Time => {
                if self.time_index > 0 {
                    self.time_index -= 1;
                }
            }
        }
    }

    pub fn move_right(&mut self) {
        match self.active_row {
            ActiveRow::Wordset => {
                if self.wordset_index < self.wordset_options.len() - 1 {
                    self.wordset_index += 1;
                }
            }
            ActiveRow::Time => {
                if self.time_index < self.time_options.len() - 1 {
                    self.time_index += 1;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.active_row = ActiveRow::Wordset;
        self.wordset_index = 0;
        self.time_index = 0;
    }
}

impl Widget for &ModeSelectWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 9;
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let title = Paragraph::new(Line::from("Select Mode").style(Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center);
        title.render(Rect { x: area.x, y: start_y, width: area.width, height: 1 }, buf);

        let wordset_label = Paragraph::new(Line::from("Wordset:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        wordset_label.render(Rect { x: area.x, y: start_y + 2, width: area.width, height: 1 }, buf);

        let wordset_value_style = match self.active_row {
            ActiveRow::Wordset => Style::default().fg(Color::Yellow),
            ActiveRow::Time => Style::default().fg(Color::White),
        };
        let wordset_value = Line::from(format!("< {} >", self.wordset_options[self.wordset_index])).style(wordset_value_style);
        Paragraph::new(wordset_value).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 3, width: area.width, height: 1 }, buf
        );

        let time_label = Paragraph::new(Line::from("Time:").style(Style::default().fg(Color::White)))
            .alignment(Alignment::Center);
        time_label.render(Rect { x: area.x, y: start_y + 5, width: area.width, height: 1 }, buf);

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
            ActiveRow::Wordset => Style::default().fg(Color::White),
        };
        let time_value = Line::from(format!("< {} >", time_str)).style(time_value_style);
        Paragraph::new(time_value).alignment(Alignment::Center).render(
            Rect { x: area.x, y: start_y + 6, width: area.width, height: 1 }, buf
        );

        let hint = Paragraph::new(
            Line::from("↑ ↓ : select row  ← → : change value  Enter : start  Esc : back").style(Style::default().fg(Color::DarkGray))
        ).alignment(Alignment::Center);
        hint.render(Rect { x: area.x, y: start_y + 8, width: area.width, height: 1 }, buf);
    }
}
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSection {
    QuickStart,
    Translation,
    Typing,
}

#[derive(Debug)]
pub struct SettingsWidget {
    active_row: usize,
    section: SettingsSection,
    mode_index: usize,
    quick_start_wordset_index: usize,
    quick_start_translation_set_index: usize,
    quick_start_time_index: usize,
    translation_set_index: usize,
    translation_time_index: usize,
    typing_layout_conversion: bool,
    mode_options: Vec<String>,
    wordset_options: Vec<String>,
    translation_set_options: Vec<String>,
    time_options: Vec<u32>,
}

impl SettingsWidget {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wordset_names: Vec<String>,
        translation_set_names: Vec<String>,
        current_mode: &str,
        current_quick_start_wordset: &str,
        current_quick_start_translation_set: &str,
        current_quick_start_time: u32,
        current_translation_set: &str,
        current_translation_time: u32,
        typing_layout_conversion: bool,
    ) -> Self {
        let times: Vec<u32> = vec![15, 30, 45, 60, 90, 120, 180, 300];
        let mode_options = vec!["typing".to_string(), "translation".to_string()];

        let mode_index = mode_options
            .iter()
            .position(|m| m == current_mode)
            .unwrap_or(0);

        let quick_start_wordset_index = wordset_names
            .iter()
            .position(|n| n == current_quick_start_wordset)
            .unwrap_or(0);

        let quick_start_translation_set_index = translation_set_names
            .iter()
            .position(|n| n == current_quick_start_translation_set)
            .unwrap_or(0);

        let quick_start_time_index = times
            .iter()
            .position(|&t| t == current_quick_start_time)
            .unwrap_or(0);

        let translation_set_index = translation_set_names
            .iter()
            .position(|n| n == current_translation_set)
            .unwrap_or(0);

        let translation_time_index = times
            .iter()
            .position(|&t| t == current_translation_time)
            .unwrap_or(0);

        Self {
            active_row: 0,
            section: SettingsSection::QuickStart,
            mode_index,
            quick_start_wordset_index,
            quick_start_translation_set_index,
            quick_start_time_index,
            translation_set_index,
            translation_time_index,
            typing_layout_conversion,
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
                let max = self.row_count().saturating_sub(1);
                self.active_row = if self.active_row > 0 {
                    self.active_row - 1
                } else {
                    max
                };
                None
            }
            KeyCode::Down => {
                let max = self.row_count().saturating_sub(1);
                self.active_row = if self.active_row < max {
                    self.active_row + 1
                } else {
                    0
                };
                None
            }
            KeyCode::Left => {
                self.adjust_row_value(-1);
                None
            }
            KeyCode::Right => {
                self.adjust_row_value(1);
                None
            }
            KeyCode::Enter => Some(SettingsAction::Save),
            KeyCode::Esc => Some(SettingsAction::Exit),
            _ => None,
        }
    }

    fn row_count(&self) -> usize {
        match self.section {
            SettingsSection::QuickStart => 4,
            SettingsSection::Translation => 3,
            SettingsSection::Typing => 2,
        }
    }

    fn switch_section(&mut self, delta: i32) {
        self.section = match (self.section, delta) {
            (SettingsSection::QuickStart, d) if d > 0 => SettingsSection::Translation,
            (SettingsSection::Translation, d) if d > 0 => SettingsSection::Typing,
            (SettingsSection::Translation, d) if d < 0 => SettingsSection::QuickStart,
            (SettingsSection::Typing, d) if d < 0 => SettingsSection::Translation,
            (s, _) => s,
        };
        let max = self.row_count().saturating_sub(1);
        if self.active_row > max {
            self.active_row = max;
        }
    }

    fn adjust_index(idx: &mut usize, len: usize, delta: i32) {
        if len == 0 {
            return;
        }
        if delta < 0 {
            if *idx > 0 {
                *idx -= 1;
            }
        } else if *idx + 1 < len {
            *idx += 1;
        }
    }

    fn adjust_row_value(&mut self, delta: i32) {
        match self.section {
            SettingsSection::QuickStart => match self.active_row {
                0 => self.switch_section(delta),
                1 => Self::adjust_index(&mut self.mode_index, self.mode_options.len(), delta),
                2 => {
                    if self.quick_start_mode() == "typing" {
                        Self::adjust_index(
                            &mut self.quick_start_wordset_index,
                            self.wordset_options.len(),
                            delta,
                        );
                    } else {
                        Self::adjust_index(
                            &mut self.quick_start_translation_set_index,
                            self.translation_set_options.len(),
                            delta,
                        );
                    }
                }
                3 => Self::adjust_index(
                    &mut self.quick_start_time_index,
                    self.time_options.len(),
                    delta,
                ),
                _ => {}
            },
            SettingsSection::Translation => match self.active_row {
                0 => self.switch_section(delta),
                1 => Self::adjust_index(
                    &mut self.translation_set_index,
                    self.translation_set_options.len(),
                    delta,
                ),
                2 => Self::adjust_index(
                    &mut self.translation_time_index,
                    self.time_options.len(),
                    delta,
                ),
                _ => {}
            },
            SettingsSection::Typing => {
                if self.active_row == 0 {
                    self.switch_section(delta);
                } else if self.active_row == 1 {
                    self.typing_layout_conversion = delta > 0;
                }
            }
        }
    }

    pub fn typing_layout_conversion(&self) -> bool {
        self.typing_layout_conversion
    }

    pub fn quick_start_mode(&self) -> &str {
        self.mode_options
            .get(self.mode_index)
            .map(String::as_str)
            .unwrap_or("typing")
    }

    pub fn quick_start_wordset(&self) -> &str {
        self.wordset_options
            .get(self.quick_start_wordset_index)
            .map(String::as_str)
            .unwrap_or("en_1000")
    }

    pub fn quick_start_translation_set(&self) -> &str {
        self.translation_set_options
            .get(self.quick_start_translation_set_index)
            .map(String::as_str)
            .unwrap_or("ru_en_a1")
    }

    pub fn quick_start_time(&self) -> u32 {
        *self
            .time_options
            .get(self.quick_start_time_index)
            .unwrap_or(&60)
    }

    pub fn translation_set(&self) -> &str {
        self.translation_set_options
            .get(self.translation_set_index)
            .map(String::as_str)
            .unwrap_or("ru_en_a1")
    }

    pub fn translation_time(&self) -> u32 {
        *self
            .time_options
            .get(self.translation_time_index)
            .unwrap_or(&60)
    }
}

fn format_time(seconds: u32) -> String {
    if seconds >= 60 {
        let min = seconds / 60;
        let sec = seconds % 60;
        if sec == 0 {
            format!("{} min", min)
        } else {
            format!("{} min {} sec", min, sec)
        }
    } else {
        format!("{} sec", seconds)
    }
}

impl Widget for &SettingsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 14;
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let title = Paragraph::new(Line::from("Settings").style(Style::default().fg(Color::Cyan)))
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

        let section_label =
            Paragraph::new(Line::from("Configure:").style(Style::default().fg(Color::White)))
                .alignment(Alignment::Center);
        section_label.render(
            Rect {
                x: area.x,
                y: start_y + 2,
                width: area.width,
                height: 1,
            },
            buf,
        );

        let section_text = match self.section {
            SettingsSection::QuickStart => "Quick Start",
            SettingsSection::Translation => "Translation",
            SettingsSection::Typing => "Typing",
        };
        let section_style = if self.active_row == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };
        Paragraph::new(Line::from(format!("< {} >", section_text)).style(section_style))
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

        match self.section {
            SettingsSection::QuickStart => {
                Paragraph::new(Line::from("Mode:").style(Style::default().fg(Color::White)))
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

                let mode_style = if self.active_row == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(
                    Line::from(format!("< {} >", self.quick_start_mode())).style(mode_style),
                )
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

                let target_label = if self.quick_start_mode() == "typing" {
                    "Wordset:"
                } else {
                    "Translation set:"
                };
                Paragraph::new(Line::from(target_label).style(Style::default().fg(Color::White)))
                    .alignment(Alignment::Center)
                    .render(
                        Rect {
                            x: area.x,
                            y: start_y + 7,
                            width: area.width,
                            height: 1,
                        },
                        buf,
                    );

                let target_value = if self.quick_start_mode() == "typing" {
                    self.quick_start_wordset().to_string()
                } else {
                    self.quick_start_translation_set().to_string()
                };
                let target_style = if self.active_row == 2 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(Line::from(format!("< {} >", target_value)).style(target_style))
                    .alignment(Alignment::Center)
                    .render(
                        Rect {
                            x: area.x,
                            y: start_y + 8,
                            width: area.width,
                            height: 1,
                        },
                        buf,
                    );

                Paragraph::new(Line::from("Time:").style(Style::default().fg(Color::White)))
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

                let time_style = if self.active_row == 3 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(
                    Line::from(format!("< {} >", format_time(self.quick_start_time())))
                        .style(time_style),
                )
                .alignment(Alignment::Center)
                .render(
                    Rect {
                        x: area.x,
                        y: start_y + 10,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
            }
            SettingsSection::Translation => {
                Paragraph::new(Line::from("Dataset:").style(Style::default().fg(Color::White)))
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

                let set_style = if self.active_row == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(
                    Line::from(format!("< {} >", self.translation_set())).style(set_style),
                )
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

                Paragraph::new(Line::from("Time:").style(Style::default().fg(Color::White)))
                    .alignment(Alignment::Center)
                    .render(
                        Rect {
                            x: area.x,
                            y: start_y + 8,
                            width: area.width,
                            height: 1,
                        },
                        buf,
                    );

                let time_style = if self.active_row == 2 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(
                    Line::from(format!("< {} >", format_time(self.translation_time())))
                        .style(time_style),
                )
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
            }
            SettingsSection::Typing => {
                Paragraph::new(
                    Line::from("Layout conversion:").style(Style::default().fg(Color::White)),
                )
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

                let value = if self.typing_layout_conversion {
                    "On"
                } else {
                    "Off"
                };
                let style = if self.active_row == 1 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Paragraph::new(Line::from(format!("< {value} >")).style(style))
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
            }
        }

        let hint = Paragraph::new(
            Line::from("↑ ↓ : select row  ← → : change value  Enter : save  Esc : back")
                .style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);
        hint.render(
            Rect {
                x: area.x,
                y: start_y + 13,
                width: area.width,
                height: 1,
            },
            buf,
        );
    }
}

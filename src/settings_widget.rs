use crate::configuration::Settings;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};
#[derive(Debug, PartialEq, Clone)]
pub enum SettingsAction {
    Save,
    Exit,
}
#[derive(Debug)]
pub struct SettingsWidget {
    active_row: usize,
    section: usize,
    pub draft: Settings,
    wordsets: Vec<String>,
    translations: Vec<String>,
    spanish: Vec<String>,
}
impl SettingsWidget {
    pub fn from_settings(
        wordsets: Vec<String>,
        translations: Vec<String>,
        settings: &Settings,
    ) -> Self {
        let spanish = translations
            .iter()
            .filter(|n| n.starts_with("ru_es_"))
            .cloned()
            .collect();
        Self {
            active_row: 0,
            section: 0,
            draft: settings.clone(),
            wordsets,
            translations,
            spanish,
        }
    }
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        let rows = if self.section == 0 {
            4
        } else if self.section == 2 {
            2
        } else {
            3
        };
        match key.code {
            KeyCode::Up => self.active_row = (self.active_row + rows - 1) % rows,
            KeyCode::Down => self.active_row = (self.active_row + 1) % rows,
            KeyCode::Left => self.adjust(-1),
            KeyCode::Right => self.adjust(1),
            KeyCode::Enter => return Some(SettingsAction::Save),
            KeyCode::Esc => return Some(SettingsAction::Exit),
            _ => {}
        }
        None
    }
    fn adjust(&mut self, delta: isize) {
        if self.active_row == 0 {
            self.section = (self.section as isize + delta).clamp(0, 3) as usize;
            return;
        }
        match (self.section, self.active_row) {
            (0, 1) => cycle_string(
                &mut self.draft.quick_start_mode,
                &["typing".into(), "translation".into(), "dictation".into()],
                delta,
            ),
            (0, 2) => match self.draft.quick_start_mode.as_str() {
                "typing" => {
                    cycle_string(&mut self.draft.quick_start_wordset, &self.wordsets, delta)
                }
                "dictation" => cycle_string(
                    &mut self.draft.quick_start_dictation_set,
                    &self.spanish,
                    delta,
                ),
                _ => cycle_string(
                    &mut self.draft.quick_start_translation_set,
                    &self.translations,
                    delta,
                ),
            },
            (0, 3) => {
                if self.draft.quick_start_mode == "dictation" {
                    cycle_number(
                        &mut self.draft.quick_start_dictation_length,
                        &[10, 20, 50],
                        delta,
                    )
                } else {
                    cycle_number(&mut self.draft.quick_start_time, TIMES, delta)
                }
            }
            (1, 1) => cycle_string(&mut self.draft.translation_set, &self.translations, delta),
            (1, 2) => cycle_number(&mut self.draft.translation_time, TIMES, delta),
            (2, 1) => self.draft.typing_layout_conversion = delta > 0,
            (3, 1) => cycle_string(&mut self.draft.dictation_set, &self.spanish, delta),
            (3, 2) => cycle_number(&mut self.draft.dictation_length, &[10, 20, 50], delta),
            _ => {}
        }
    }
}
const TIMES: &[u32] = &[15, 30, 45, 60, 90, 120, 180, 300];
fn cycle_string(value: &mut String, options: &[String], delta: isize) {
    if !options.is_empty() {
        let index = options.iter().position(|o| o == value).unwrap_or(0);
        *value =
            options[(index as isize + delta).clamp(0, options.len() as isize - 1) as usize].clone();
    }
}
fn cycle_number(value: &mut u32, options: &[u32], delta: isize) {
    let index = options.iter().position(|o| o == value).unwrap_or(0);
    *value = options[(index as isize + delta).clamp(0, options.len() as isize - 1) as usize];
}
fn format_time(seconds: u32) -> String {
    if seconds >= 60 {
        if seconds.is_multiple_of(60) {
            format!("{} min", seconds / 60)
        } else {
            format!("{} min {} sec", seconds / 60, seconds % 60)
        }
    } else {
        format!("{seconds} sec")
    }
}
impl Widget for &SettingsWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let y = area.y + area.height.saturating_sub(14) / 2;
        let section = ["Quick Start", "Translation", "Typing", "Dictation"][self.section];
        let mut lines = vec![
            (0, "Settings".into(), None),
            (2, "Configure:".into(), None),
            (3, format!("< {section} >"), Some(0)),
        ];
        match self.section {
            0 => {
                let (label, target) = match self.draft.quick_start_mode.as_str() {
                    "typing" => ("Wordset:", &self.draft.quick_start_wordset),
                    "dictation" => ("Spanish set:", &self.draft.quick_start_dictation_set),
                    _ => ("Translation set:", &self.draft.quick_start_translation_set),
                };
                let dictation = self.draft.quick_start_mode == "dictation";
                lines.extend([
                    (5, "Mode:".into(), None),
                    (6, format!("< {} >", self.draft.quick_start_mode), Some(1)),
                    (7, label.into(), None),
                    (8, format!("< {target} >"), Some(2)),
                    (
                        9,
                        if dictation {
                            "Words:".into()
                        } else {
                            "Time:".into()
                        },
                        None,
                    ),
                    (
                        10,
                        format!(
                            "< {} >",
                            if dictation {
                                self.draft.quick_start_dictation_length.to_string()
                            } else {
                                format_time(self.draft.quick_start_time)
                            }
                        ),
                        Some(3),
                    ),
                ]);
            }
            1 | 3 => {
                let (set, value, label) = if self.section == 1 {
                    (
                        &self.draft.translation_set,
                        format_time(self.draft.translation_time),
                        "Time:",
                    )
                } else {
                    (
                        &self.draft.dictation_set,
                        self.draft.dictation_length.to_string(),
                        "Words:",
                    )
                };
                lines.extend([
                    (5, "Dataset:".into(), None),
                    (6, format!("< {set} >"), Some(1)),
                    (8, label.into(), None),
                    (9, format!("< {value} >"), Some(2)),
                ]);
            }
            _ => lines.extend([
                (5, "Layout conversion:".into(), None),
                (
                    6,
                    format!(
                        "< {} >",
                        if self.draft.typing_layout_conversion {
                            "On"
                        } else {
                            "Off"
                        }
                    ),
                    Some(1),
                ),
            ]),
        }
        lines.push((
            13,
            "↑ ↓ : select row  ← → : change value  Enter : save  Esc : back".into(),
            None,
        ));
        for (offset, text, row) in lines {
            if y + offset < area.bottom() {
                Paragraph::new(Line::from(text).style(Style::default().fg(
                    if row == Some(self.active_row) {
                        Color::Yellow
                    } else {
                        Color::White
                    },
                )))
                .alignment(Alignment::Center)
                .render(Rect::new(area.x, y + offset, area.width, 1), buf);
            }
        }
    }
}

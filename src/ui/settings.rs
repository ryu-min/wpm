use crate::domain::Settings;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SettingsAction {
    Save,
    Exit,
}
#[derive(Debug)]
pub(crate) struct SettingsWidget {
    active_row: usize,
    section: usize,
    pub(crate) draft: Settings,
    wordsets: Vec<String>,
    translations: Vec<String>,
}
impl SettingsWidget {
    pub(crate) fn from_settings(
        wordsets: Vec<String>,
        translations: Vec<String>,
        settings: &Settings,
    ) -> Self {
        Self {
            active_row: 0,
            section: 0,
            draft: settings.clone(),
            wordsets,
            translations,
        }
    }
    pub(crate) fn handle_input(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        let rows = if self.section == 0 || self.section == 1 {
            4
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
            self.section = (self.section as isize + delta).clamp(0, 2) as usize;
            return;
        }
        match (self.section, self.active_row) {
            (0, 1) => cycle_string(
                &mut self.draft.quick_start_mode,
                &["typing".into(), "translation".into()],
                delta,
            ),
            (0, 2) => match self.draft.quick_start_mode.as_str() {
                "typing" => {
                    cycle_string(&mut self.draft.quick_start_wordset, &self.wordsets, delta)
                }
                _ => cycle_string(
                    &mut self.draft.quick_start_translation_set,
                    &self.translations,
                    delta,
                ),
            },
            (0, 3) => cycle_number(&mut self.draft.quick_start_time, TIMES, delta),
            (1, 1) => cycle_string(&mut self.draft.translation_set, &self.translations, delta),
            (1, 2) => cycle_number(&mut self.draft.translation_time, TIMES, delta),
            (1, 3) => self.draft.translation_ignore_case = delta > 0,
            (2, 2) => self.draft.typing_ignore_case = delta > 0,
            (2, 1) => self.draft.typing_layout_conversion = delta > 0,
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
        let section = ["Quick Start", "Translation", "Typing"][self.section];
        let mut lines = vec![
            (0, "Settings".into(), None),
            (2, "Configure:".into(), None),
            (3, format!("< {section} >"), Some(0)),
        ];
        match self.section {
            0 => {
                let (label, target) = match self.draft.quick_start_mode.as_str() {
                    "typing" => ("Wordset:", &self.draft.quick_start_wordset),
                    _ => ("Translation set:", &self.draft.quick_start_translation_set),
                };
                lines.extend([
                    (5, "Mode:".into(), None),
                    (6, format!("< {} >", self.draft.quick_start_mode), Some(1)),
                    (7, label.into(), None),
                    (8, format!("< {target} >"), Some(2)),
                    (9, "Time:".into(), None),
                    (
                        10,
                        format!("< {} >", format_time(self.draft.quick_start_time)),
                        Some(3),
                    ),
                ]);
            }
            1 => {
                let set = &self.draft.translation_set;
                let value = format_time(self.draft.translation_time);
                lines.extend([
                    (5, "Dataset:".into(), None),
                    (6, format!("< {set} >"), Some(1)),
                    (8, "Time:".into(), None),
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
        if self.section == 1 || self.section == 2 {
            let enabled = if self.section == 1 {
                self.draft.translation_ignore_case
            } else {
                self.draft.typing_ignore_case
            };
            let (offset, row) = if self.section == 1 { (10, 3) } else { (8, 2) };
            lines.extend([
                (offset, "Ignore case:".into(), None),
                (
                    offset + 1,
                    format!("< {} >", if enabled { "On" } else { "Off" }),
                    Some(row),
                ),
            ]);
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

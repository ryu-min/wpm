use crate::domain::{self as language_input, TypingSession};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TypingAction {
    Exit,
}

pub(crate) struct TypingWidget<'a> {
    session: &'a TypingSession,
}
impl<'a> TypingWidget<'a> {
    pub(crate) fn new(session: &'a TypingSession) -> Self {
        Self { session }
    }
    pub(crate) fn handle_input(session: &mut TypingSession, key: KeyEvent) -> Option<TypingAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char(ch) => {
                session.add_char(ch);
                None
            }
            KeyCode::Backspace => {
                session.remove_char();
                None
            }
            KeyCode::Enter => {
                session.insert_newline_if_expected();
                None
            }
            KeyCode::Esc => Some(TypingAction::Exit),
            _ => None,
        }
    }

    pub(crate) fn extend_result(session: &mut TypingSession, key: KeyEvent) -> bool {
        let character = match key.code {
            crossterm::event::KeyCode::Char(ch) => Some(ch),
            _ => None,
        };
        session.extend_result(character)
    }
}

impl TypingWidget<'_> {
    fn split_text_into_lines(&self, text: &str, max_width: usize) -> Vec<(usize, usize)> {
        let mut lines = Vec::new();
        let chars = language_input::units(text);
        let mut start = 0;

        while start < chars.len() {
            if chars[start] == "\n" {
                lines.push((start, start + 1));
                start += 1;
                continue;
            }

            let mut end = start + max_width.min(chars.len() - start);
            let mut newline_pos = None;

            for (i, ch) in chars.iter().enumerate().take(end).skip(start) {
                if ch == "\n" {
                    newline_pos = Some(i + 1);
                    break;
                }
            }

            if let Some(nl) = newline_pos {
                end = nl;
            } else if end < chars.len()
                && let Some(last_space) = chars[start..end]
                    .iter()
                    .rposition(|ch| ch.chars().all(char::is_whitespace))
                    .map(|idx| start + idx + 1)
            {
                end = last_space;
            }

            lines.push((start, end));
            start = end;
        }

        lines
    }

    fn get_current_line_index(&self, line_ranges: &[(usize, usize)], input_len: usize) -> usize {
        for (idx, &(start, end)) in line_ranges.iter().enumerate() {
            if input_len >= start && input_len <= end {
                if input_len == end && idx + 1 < line_ranges.len() {
                    return idx + 1;
                }
                return idx;
            }
        }
        line_ranges.len().saturating_sub(1)
    }
}

impl Widget for TypingWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let available_width = area.width.saturating_sub(2) as usize;

        if available_width == 0 {
            return;
        }

        let line_width = (available_width as f64 * 0.7) as usize;
        let line_ranges = self.split_text_into_lines(self.session.target_text(), line_width);

        if line_ranges.is_empty() {
            return;
        }

        let input_len = language_input::units(self.session.get_input()).len();
        let current_line_idx = self.get_current_line_index(&line_ranges, input_len);

        let target_chars = language_input::units(self.session.target_text());
        let input_chars = language_input::units(self.session.get_input());

        let wpm_str = format!("{} wpm", self.session.displayed_wpm() as u32);
        let time_str = format!("{:.1}s", self.session.displayed_elapsed());

        let wpm_line = Line::from(wpm_str).style(Style::default().fg(Color::Cyan));
        let time_line = Line::from(time_str).style(Style::default().fg(Color::Yellow));

        let wpm_area = Rect {
            x: area.x,
            y: area.y,
            width: 10,
            height: 1,
        };

        let time_area = Rect {
            x: area.x.saturating_add(area.width).saturating_sub(8),
            y: area.y,
            width: 8,
            height: 1,
        };

        Paragraph::new(wpm_line)
            .alignment(Alignment::Left)
            .render(wpm_area, buf);

        Paragraph::new(time_line)
            .alignment(Alignment::Right)
            .render(time_area, buf);

        let start_idx = if current_line_idx == 0 {
            0
        } else if current_line_idx >= line_ranges.len() - 1 {
            line_ranges.len().saturating_sub(3)
        } else {
            current_line_idx.saturating_sub(1)
        };

        let num_lines = 3.min(line_ranges.len());
        let text_area_y = area.y + area.height.saturating_sub(1) / 2 - num_lines as u16 / 2;

        for (idx, &(start, end)) in line_ranges.iter().enumerate().skip(start_idx).take(3) {
            let line_length = end - start;
            let y = text_area_y + idx.saturating_sub(start_idx) as u16;

            if y < area.y || y >= area.y + area.height {
                continue;
            }

            let is_current = idx == current_line_idx;

            let spans: Vec<Span> = (0..line_length)
                .map(|i| {
                    let global_idx = start + i;
                    if global_idx < target_chars.len() {
                        let target_char = &target_chars[global_idx];
                        if global_idx < input_chars.len() {
                            let input_char = &input_chars[global_idx];
                            if self.session.accepts(global_idx, input_char, target_char) {
                                Span::styled(
                                    target_char.to_string(),
                                    Style::default().fg(Color::Green),
                                )
                            } else {
                                Span::styled(
                                    target_char.to_string(),
                                    Style::default()
                                        .fg(Color::Red)
                                        .add_modifier(Modifier::CROSSED_OUT),
                                )
                            }
                        } else if global_idx == input_chars.len() && is_current {
                            Span::styled(
                                target_char.to_string(),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::UNDERLINED),
                            )
                        } else {
                            Span::styled(target_char.to_string(), Style::default().fg(Color::White))
                        }
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();

            let line = Line::from(spans);
            let line_x = area.x + ((area.width as usize - line_length) / 2) as u16;

            let line_area = Rect {
                x: line_x,
                y,
                width: line_length as u16,
                height: 1,
            };

            Paragraph::new(line)
                .alignment(Alignment::Left)
                .render(line_area, buf);
        }
    }
}

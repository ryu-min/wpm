use crate::language_input::{self, Language};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub enum TypingAction {
    Exit,
}

#[derive(Debug)]
pub struct TypingWidget {
    target_text: String,
    input_text: String,
    start_time: Option<std::time::Instant>,
    pub wpm: f64,
    pub elapsed: f64,
    time_limit: Option<u64>,
    last_stats_update: Option<std::time::Instant>,
    layout_conversion: bool,
    languages: Vec<Language>,
    finished_elapsed: Option<f64>,
    extend_final_grapheme: bool,
}

const STATS_UPDATE_INTERVAL_MS: u64 = 100;

impl TypingWidget {
    pub fn new(target_text: String) -> Self {
        Self {
            target_text,
            input_text: String::new(),
            start_time: None,
            wpm: 0.0,
            elapsed: 0.0,
            time_limit: None,
            last_stats_update: None,
            layout_conversion: false,
            languages: Vec::new(),
            finished_elapsed: None,
            extend_final_grapheme: false,
        }
    }

    pub fn with_time_limit(mut self, seconds: u64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    pub fn with_layout_conversion(mut self, enabled: bool) -> Self {
        self.layout_conversion = enabled;
        self
    }

    pub fn with_languages(mut self, languages: Vec<Language>) -> Self {
        self.languages = languages;
        self
    }

    fn policy(&self, index: usize) -> Language {
        self.languages
            .get(index)
            .copied()
            .unwrap_or(if self.layout_conversion {
                Language::Layout
            } else {
                Language::Exact
            })
    }

    fn correct_count(&self) -> usize {
        language_input::units(&self.input_text)
            .iter()
            .zip(language_input::units(&self.target_text))
            .enumerate()
            .filter(|(index, (a, b))| language_input::equivalent(a, b, self.policy(*index)))
            .count()
    }

    pub fn with_target_text(mut self, target_text: String) -> Self {
        self.target_text = target_text;
        self.input_text.clear();
        self
    }

    pub fn set_target_text(&mut self, target_text: String) {
        self.target_text = target_text;
        self.input_text.clear();
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Option<TypingAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Char(ch) => {
                self.add_char(ch);
                None
            }
            KeyCode::Backspace => {
                self.remove_char();
                None
            }
            KeyCode::Enter => {
                if language_input::units(&self.target_text)
                    .get(language_input::units(&self.input_text).len())
                    .map(String::as_str)
                    == Some("\n")
                {
                    self.add_char('\n');
                }
                None
            }
            KeyCode::Esc => Some(TypingAction::Exit),
            _ => None,
        }
    }

    pub fn add_char(&mut self, ch: char) {
        self.input_text.push(ch);
        self.start_timer_if_needed();
    }

    pub fn remove_char(&mut self) {
        language_input::pop_grapheme(&mut self.input_text);
    }

    pub fn reset(&mut self) {
        self.input_text.clear();
        self.start_time = None;
        self.wpm = 0.0;
        self.elapsed = 0.0;
        self.last_stats_update = None;
        self.finished_elapsed = None;
        self.extend_final_grapheme = false;
    }

    pub fn update_stats(&mut self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_stats_update
            && now.duration_since(last).as_millis() < STATS_UPDATE_INTERVAL_MS as u128
        {
            return;
        }
        self.last_stats_update = Some(now);
        self.elapsed = self.get_elapsed_time();
        self.wpm = self.get_wpm();
    }

    pub fn get_wpm(&self) -> f64 {
        let elapsed = self.get_elapsed_time();
        if elapsed == 0.0 {
            return 0.0;
        }
        let correct = self.correct_count() as f64;
        if correct == 0.0 {
            return 0.0;
        }
        let words = correct / 5.0;
        (words / elapsed) * 60.0
    }

    pub fn get_elapsed_time(&self) -> f64 {
        if let Some(elapsed) = self.finished_elapsed {
            return elapsed;
        }
        match self.start_time {
            Some(time) => time.elapsed().as_secs_f64(),
            None => 0.0,
        }
    }

    fn start_timer_if_needed(&mut self) {
        if self.start_time.is_none() && !self.input_text.is_empty() {
            self.start_time = Some(std::time::Instant::now());
        }
    }

    pub fn get_input(&self) -> &str {
        &self.input_text
    }

    fn timed_out(&self) -> bool {
        self.time_limit
            .zip(self.start_time)
            .is_some_and(|(limit, start)| start.elapsed().as_secs() >= limit)
    }

    pub fn is_complete(&self) -> bool {
        self.timed_out()
            || language_input::units(&self.input_text).len()
                >= language_input::units(&self.target_text).len()
    }

    pub fn finalize(&mut self) {
        if self.finished_elapsed.is_none() {
            self.extend_final_grapheme = !self.timed_out() && !self.input_text.is_empty();
            self.finished_elapsed = Some(self.get_elapsed_time());
        }
    }

    /// A combining mark can arrive in another terminal event after auto-completion.
    /// Any other key seals the result and retains the normal Result behavior.
    pub fn extend_result(&mut self, key: KeyEvent) -> bool {
        if self.extend_final_grapheme
            && let crossterm::event::KeyCode::Char(ch) = key.code
            && unicode_normalization::char::is_combining_mark(ch)
        {
            let before = language_input::units(&self.input_text).len();
            let mut candidate = self.input_text.clone();
            candidate.push(ch);
            if language_input::units(&candidate).len() == before {
                self.input_text = candidate;
                return true;
            }
        }
        self.extend_final_grapheme = false;
        false
    }

    pub fn get_accuracy(&self) -> f64 {
        if self.input_text.is_empty() {
            return 0.0;
        }
        let total = language_input::units(&self.input_text).len();
        if total == 0 {
            return 0.0;
        }
        let correct = self.correct_count();
        correct as f64 / total as f64
    }
}

impl TypingWidget {
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

impl Widget for &TypingWidget {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let available_width = area.width.saturating_sub(2) as usize;

        if available_width == 0 {
            return;
        }

        let line_width = (available_width as f64 * 0.7) as usize;
        let line_ranges = self.split_text_into_lines(&self.target_text, line_width);

        if line_ranges.is_empty() {
            return;
        }

        let input_len = language_input::units(&self.input_text).len();
        let current_line_idx = self.get_current_line_index(&line_ranges, input_len);

        let target_chars = language_input::units(&self.target_text);
        let input_chars = language_input::units(&self.input_text);

        let wpm_str = format!("{} wpm", self.wpm as u32);
        let time_str = format!("{:.1}s", self.elapsed);

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
                            if language_input::equivalent(
                                input_char,
                                target_char,
                                self.policy(global_idx),
                            ) {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_positions_backspace_and_stats_share_graphemes() {
        let mut composed = TypingWidget::new("caféx".into())
            .with_layout_conversion(true)
            .with_languages(vec![Language::Spanish; 5]);
        let mut decomposed = TypingWidget::new("caféx".into())
            .with_layout_conversion(true)
            .with_languages(vec![Language::Spanish; 5]);
        for ch in "café".chars() {
            composed.add_char(ch);
        }
        for ch in "cafe\u{301}".chars() {
            decomposed.add_char(ch);
        }
        assert_eq!(composed.correct_count(), decomposed.correct_count());
        assert_eq!(decomposed.correct_count(), 4);
        decomposed.remove_char();
        assert_eq!(decomposed.get_input(), "caf");
        decomposed.add_char('e');
        decomposed.add_char('\u{301}');
        decomposed.add_char('x');
        assert!(decomposed.is_complete());
        assert_eq!(decomposed.get_accuracy(), 1.0);
    }
}

#[cfg(test)]
mod final_grapheme_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    #[test]
    fn late_marks_recompute_numerator_with_frozen_time_and_other_keys_seal() {
        let mut typing = TypingWidget::new("café".into());
        for ch in "cafe".chars() {
            typing.add_char(ch);
        }
        typing.start_time = Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
        typing.finalize();
        let elapsed = typing.get_elapsed_time();
        let wpm = typing.get_wpm();
        assert!(typing.extend_result(key(KeyCode::Char('\u{301}'))));
        assert_eq!(typing.get_accuracy(), 1.0);
        assert_eq!(typing.get_elapsed_time(), elapsed);
        assert!(typing.get_wpm() > wpm);
        assert!(!typing.extend_result(key(KeyCode::Backspace)));
        let input = typing.get_input().to_string();
        assert!(!typing.extend_result(key(KeyCode::Char('\u{308}'))));
        assert_eq!(typing.get_input(), input);
        typing.reset();
        assert_eq!(typing.finished_elapsed, None);
    }
    #[test]
    fn timed_result_and_arrow_prevent_late_marks() {
        for timed in [false, true] {
            let mut typing = TypingWidget::new("café".into()).with_time_limit(1);
            for ch in "cafe".chars() {
                typing.add_char(ch);
            }
            if timed {
                typing.start_time =
                    Some(std::time::Instant::now() - std::time::Duration::from_secs(2));
            }
            typing.finalize();
            let before = (
                typing.get_input().to_string(),
                typing.get_accuracy(),
                typing.get_wpm(),
                typing.get_elapsed_time(),
            );
            if !timed {
                assert!(!typing.extend_result(key(KeyCode::Right)));
            }
            assert!(!typing.extend_result(key(KeyCode::Char('\u{301}'))));
            assert_eq!(
                (
                    typing.get_input().to_string(),
                    typing.get_accuracy(),
                    typing.get_wpm(),
                    typing.get_elapsed_time()
                ),
                before
            );
        }
    }
}

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
}

impl TypingWidget {
    pub fn new(target_text: String) -> Self {
        Self {
            target_text,
            input_text: String::new(),
            start_time: None,
            wpm: 0.0,
            elapsed: 0.0,
            time_limit: None,
        }
    }

    pub fn with_time_limit(mut self, seconds: u64) -> Self {
        self.time_limit = Some(seconds);
        self
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
            KeyCode::Esc => Some(TypingAction::Exit),
            _ => None,
        }
    }

    pub fn add_char(&mut self, ch: char) {
        self.input_text.push(ch);
        self.start_timer_if_needed();
    }

    pub fn remove_char(&mut self) {
        self.input_text.pop();
    }

    pub fn reset(&mut self) {
        self.input_text.clear();
        self.start_time = None;
        self.wpm = 0.0;
        self.elapsed = 0.0;
    }

    pub fn update_stats(&mut self) {
        self.elapsed = self.get_elapsed_time();
        self.wpm = self.get_wpm();
    }

    pub fn get_wpm(&self) -> f64 {
        let elapsed = match self.start_time {
            Some(time) => time.elapsed().as_secs_f64(),
            None => return 0.0,
        };
        if elapsed == 0.0 {
            return 0.0;
        }
        let correct = self.input_text.chars()
            .zip(self.target_text.chars())
            .filter(|(a, b)| a == b)
            .count() as f64;
        if correct == 0.0 {
            return 0.0;
        }
        let words = correct / 5.0;
        (words / elapsed) * 60.0
    }

    pub fn get_elapsed_time(&self) -> f64 {
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

    pub fn is_complete(&self) -> bool {
        if let Some(limit) = self.time_limit {
            if let Some(start) = self.start_time {
                if start.elapsed().as_secs() >= limit {
                    return true;
                }
            }
        }
        self.input_text.len() >= self.target_text.len()
    }

    pub fn get_accuracy(&self) -> f64 {
        if self.input_text.is_empty() {
            return 0.0;
        }
        let total = self.input_text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let correct = self
            .input_text
            .chars()
            .zip(self.target_text.chars())
            .filter(|(a, b)| a == b)
            .count();
        correct as f64 / total as f64
    }
}

impl TypingWidget {
    fn split_text_into_lines(&self, text: &str, max_width: usize) -> Vec<(usize, usize)> {
        let mut lines = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;
        
        while start < chars.len() {
            if chars[start] == '\n' {
                lines.push((start, start + 1));
                start += 1;
                continue;
            }
            
            let mut end = start + max_width.min(chars.len() - start);
            let mut newline_pos = None;
            
            for i in start..end {
                if chars[i] == '\n' {
                    newline_pos = Some(i + 1);
                    break;
                }
            }
            
            if let Some(nl) = newline_pos {
                end = nl;
            } else if end < chars.len() {
                let mut last_space = end;
                for i in (start..end).rev() {
                    if chars[i].is_whitespace() {
                        last_space = i + 1;
                        break;
                    }
                }
                if last_space > start {
                    end = last_space;
                }
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

        let input_len = self.input_text.chars().count();
        let current_line_idx = self.get_current_line_index(&line_ranges, input_len);
        
        let target_chars: Vec<char> = self.target_text.chars().collect();
        let input_chars: Vec<char> = self.input_text.chars().collect();
        
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
                        let target_char = target_chars[global_idx];
                        if global_idx < input_chars.len() {
                            let input_char = input_chars[global_idx];
                            if input_char == target_char {
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
                            Span::styled(
                                target_char.to_string(),
                                Style::default().fg(Color::White),
                            )
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


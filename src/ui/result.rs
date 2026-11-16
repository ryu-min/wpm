use crate::domain::TypingSession;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ResultAction {
    Restart,
    Menu,
}

#[derive(Debug, Clone)]
struct ResultItem {
    text: String,
    action: ResultAction,
}

#[derive(Debug)]
pub(crate) struct ResultWidget {
    selected_index: usize,
    items: Vec<ResultItem>,
}

impl ResultWidget {
    pub(crate) fn new() -> Self {
        Self {
            selected_index: 0,
            items: vec![
                ResultItem {
                    text: "Restart".to_string(),
                    action: ResultAction::Restart,
                },
                ResultItem {
                    text: "Menu".to_string(),
                    action: ResultAction::Menu,
                },
            ],
        }
    }

    pub(crate) fn view<'a>(&'a self, session: &'a TypingSession) -> ResultView<'a> {
        ResultView {
            widget: self,
            session,
        }
    }

    pub(crate) fn handle_input(&mut self, key: KeyEvent) -> Option<ResultAction> {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
                None
            }
            KeyCode::Enter => Some(self.items[self.selected_index].action.clone()),
            KeyCode::Esc => Some(ResultAction::Menu),
            _ => None,
        }
    }
}

pub(crate) struct ResultView<'a> {
    widget: &'a ResultWidget,
    session: &'a TypingSession,
}

impl Widget for ResultView<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let total_lines = 3 + self.widget.items.len();
        let start_y = area.y + (area.height - total_lines as u16) / 2;

        let wpm_line = Line::from(format!("WPM: {}", self.session.get_wpm() as u32))
            .style(Style::default().fg(Color::Cyan));
        let acc_line = Line::from(format!(
            "Accuracy: {:.0}%",
            self.session.get_accuracy() * 100.0
        ))
        .style(Style::default().fg(Color::Green));
        let time_line = Line::from(format!("Time: {:.1}s", self.session.get_elapsed_time()))
            .style(Style::default().fg(Color::Yellow));

        Paragraph::new(wpm_line)
            .alignment(ratatui::layout::Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
        Paragraph::new(acc_line)
            .alignment(ratatui::layout::Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 1,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
        Paragraph::new(time_line)
            .alignment(ratatui::layout::Alignment::Center)
            .render(
                Rect {
                    x: area.x,
                    y: start_y + 2,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        for (i, item) in self.widget.items.iter().enumerate() {
            let line = if i == self.widget.selected_index {
                Line::from(format!("> {}", item.text)).style(Style::default().fg(Color::Yellow))
            } else {
                Line::from(format!("  {}", item.text)).style(Style::default().fg(Color::White))
            };

            Paragraph::new(line)
                .alignment(ratatui::layout::Alignment::Center)
                .render(
                    Rect {
                        x: area.x,
                        y: start_y + 4 + i as u16,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
        }
    }
}

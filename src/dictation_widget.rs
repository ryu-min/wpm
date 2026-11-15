use crate::dictation::{Dictation, InputView, Phase};
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};
// Hidden-state rendering only receives InputView, never VocabularyEntry.
pub fn input_lines(view: InputView) -> Vec<String> {
    vec![
        "Dictation".into(),
        format!("Word {} / {}", view.index, view.total),
        view.hint.into(),
        view.status,
        format!("Input: {}", view.input),
        format!(
            "Checked: {}  Correct: {}  Errors: {}",
            view.checked,
            view.correct,
            view.checked - view.correct
        ),
        "Enter: check  F2: repeat  Esc: menu".into(),
    ]
}
impl Widget for &Dictation {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let lines = match self.phase {
            Phase::Input => input_lines(self.input_view()),
            Phase::Preparing | Phase::Unavailable => {
                vec!["Dictation".into(), self.status.clone(), "Esc: menu".into()]
            }
            Phase::Checked => vec![
                "Dictation".into(),
                self.status.clone(),
                if self.accepted {
                    "Correct".into()
                } else {
                    "Incorrect".into()
                },
                format!("Answer: {}", self.current().es),
                format!("Translation: {}", self.current().ru),
                format!("Your answer: {}", self.input),
                format!(
                    "Checked: {}  Correct: {}  Errors: {}",
                    self.checked,
                    self.correct,
                    self.errors()
                ),
                "Enter: next  F2: repeat  Esc: menu".into(),
            ],
            Phase::Finished => vec![
                "Dictation result".into(),
                format!("Checked: {}", self.checked),
                format!("Correct: {}", self.correct),
                format!("Errors: {}", self.errors()),
                format!("Correct words: {}%", self.percent()),
                "Enter: restart  Esc: menu".into(),
            ],
        };
        let y = area.y + area.height.saturating_sub(lines.len() as u16 * 2) / 2;
        for (index, line) in lines.into_iter().enumerate() {
            let row = y + index as u16 * 2;
            if row < area.bottom() {
                Paragraph::new(Line::from(line))
                    .alignment(Alignment::Center)
                    .render(Rect::new(area.x, row, area.width, 1), buf);
            }
        }
    }
}

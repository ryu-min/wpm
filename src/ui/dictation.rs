use crate::domain::{Dictation, InputView, Phase};
use ratatui::{
    layout::{Alignment, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};
// Hidden-state rendering only receives InputView, never VocabularyEntry.
pub(crate) fn input_lines(view: InputView) -> Vec<String> {
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
pub(crate) struct DictationWidget<'a>(pub(crate) &'a Dictation);
impl Widget for DictationWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let session = self.0;
        let lines = match session.phase() {
            Phase::Input => input_lines(session.input_view()),
            Phase::Preparing | Phase::Unavailable => {
                vec![
                    "Dictation".into(),
                    session.status().to_string(),
                    "Esc: menu".into(),
                ]
            }
            Phase::Checked => vec![
                "Dictation".into(),
                session.status().to_string(),
                if session.accepted() {
                    "Correct".into()
                } else {
                    "Incorrect".into()
                },
                format!("Answer: {}", session.current().es),
                format!("Translation: {}", session.current().ru),
                format!("Your answer: {}", session.input()),
                format!(
                    "Checked: {}  Correct: {}  Errors: {}",
                    session.checked(),
                    session.correct(),
                    session.errors()
                ),
                "Enter: next  F2: repeat  Esc: menu".into(),
            ],
            Phase::Finished => vec![
                "Dictation result".into(),
                format!("Checked: {}", session.checked()),
                format!("Correct: {}", session.correct()),
                format!("Errors: {}", session.errors()),
                format!("Correct words: {}%", session.percent()),
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

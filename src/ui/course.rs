use crate::domain::course::{self, CourseApp, Mode, Screen, Session};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
fn line(frame: &mut Frame, y: u16, text: impl Into<Line<'static>>, active: bool) {
    let a = frame.area();
    if y < a.bottom() {
        frame.render_widget(
            Paragraph::new(text.into())
                .alignment(Alignment::Center)
                .style(Style::default().fg(if active { Color::Yellow } else { Color::White })),
            Rect::new(a.x, y, a.width, 1),
        );
    }
}
fn clipped(text: &str, width: usize) -> String {
    if Line::from(text).width() <= width {
        return text.to_string();
    }
    let mut used = 0;
    let mut out = String::new();
    for g in course::graphemes(text) {
        let next = Line::from(g).width();
        if used + next + 1 > width {
            break;
        }
        out.push_str(g);
        used += next;
    }
    if width > 0 {
        out.push('…');
    }
    out
}
pub(crate) fn render(f: &mut Frame, app: &CourseApp) {
    let a = f.area();
    if a.height < 18 || a.width < 30 {
        line(
            f,
            a.y + a.height / 2,
            "Увеличьте окно терминала".to_string(),
            false,
        );
        return;
    }
    let y = a.y + a.height.saturating_sub(18) / 2;
    match &app.screen {
        Screen::Training(t) => {
            line(f, y, format!("Тренировка · {}", t.title), false);
            f.render_widget(
                super::TypingWidget::new(&t.typing),
                Rect::new(a.x, y + 1, a.width, a.height.saturating_sub(y - a.y + 1)),
            );
        }
        Screen::TrainingResult(t) => {
            line(f, y, "Тренировка завершена".to_string(), false);
            line(
                f,
                y + 3,
                format!("Точность печати: {:.1}%", t.typing.get_accuracy() * 100.0),
                false,
            );
            line(f, y + 5, format!("WPM: {:.1}", t.typing.get_wpm()), false);
            line(
                f,
                y + 7,
                format!("Время: {:.1} sec", t.typing.get_elapsed_time()),
                false,
            );
            line(
                f,
                y + 11,
                "Повторить тренировку".to_string(),
                app.action == 0,
            );
            line(f, y + 12, "К выбору режима".to_string(), app.action == 1);
        }

        Screen::Home => {
            line(f, y, "Испанский".to_string(), false);
            let t = app.topic();
            let range = if t.start == t.end {
                t.start.to_string()
            } else {
                format!("{}–{}", t.start, t.end)
            };
            let values = [
                format!(
                    "< {} >",
                    clipped(
                        &format!(
                            "{}. {}",
                            app.module_index() + 1,
                            app.catalog.modules[app.module_index()].title
                        ),
                        a.width.saturating_sub(4) as usize
                    )
                ),
                format!(
                    "< {} >",
                    clipped(
                        &format!("{range}. {}", t.title),
                        a.width.saturating_sub(4) as usize
                    )
                ),
                format!("< {} >", app.choice.mode.label()),
            ];
            for (i, label) in ["Модуль:", "Раздел:", "Режим:"].iter().enumerate() {
                line(f, y + 3 + i as u16 * 3, label.to_string(), false);
                line(f, y + 4 + i as u16 * 3, values[i].clone(), app.field == i);
            }
            line(
                f,
                y + 13,
                if t.entries.is_empty() {
                    t.empty_reason().into()
                } else {
                    format!("{} слов и выражений", t.entries.len())
                },
                false,
            );
            if let Some(n) = &app.notice {
                let text = clipped(n, a.width as usize);
                line(f, y + 14, text, false);
                if app.save_failed {
                    line(f, y + 15, "R — повторить сохранение".to_string(), false);
                }
            }
            line(
                f,
                y + 16,
                "↑ ↓ выбрать поле   ← → изменить".to_string(),
                false,
            );
            line(f, y + 17, "Enter начать   Esc выйти".to_string(), false);
        }
        Screen::Session(s) => session(f, y, s, app),
        Screen::Result(s) => {
            line(
                f,
                y,
                if s.mode == Mode::Exam {
                    "Экзамен завершён"
                } else {
                    "Тренировка завершена"
                }
                .to_string(),
                false,
            );
            let mistakes = s.mistakes();
            let correct = s.entries.len() - mistakes.len();
            line(
                f,
                y + 2,
                format!(
                    "Правильно {correct} из {} ({} %)",
                    s.entries.len(),
                    correct * 100 / s.entries.len()
                ),
                false,
            );
            for (i, action) in app.actions(s).iter().enumerate() {
                line(
                    f,
                    y + 11 + i as u16,
                    action.label(s.mode).to_string(),
                    app.action == i,
                );
            }
            line(
                f,
                y + 16,
                "↑ ↓ выбрать   Enter открыть   Esc назад".to_string(),
                false,
            );
        }
        Screen::Review(s) => {
            line(f, y, "Ошибки экзамена".to_string(), false);
            let errors = wrapped(result_lines(s), a.width.saturating_sub(4) as usize);
            f.render_widget(
                Paragraph::new(errors.into_iter().skip(app.scroll).collect::<Vec<_>>()),
                Rect::new(a.x + 2, y + 2, a.width.saturating_sub(4), 12),
            );
            line(
                f,
                y + 16,
                "PgUp/PgDn прокрутить   Esc к результату".to_string(),
                false,
            );
        }
    }
}
fn session(f: &mut Frame, y: u16, s: &Session, app: &CourseApp) {
    line(f, y, format!("{} · {}", s.mode.label(), s.title), false);
    line(
        f,
        y + 2,
        format!("{} из {}", s.index + 1, s.entries.len()),
        false,
    );
    let width = f.area().width.saturating_sub(4);
    let body = wrapped(session_lines(s), width as usize);
    f.render_widget(
        Paragraph::new(body.into_iter().skip(app.scroll).collect::<Vec<_>>())
            .alignment(Alignment::Center),
        Rect::new(f.area().x + 2, y + 4, width, 10),
    );
    line(
        f,
        y + 16,
        "Enter принять ответ   Esc к выбору режима".to_string(),
        false,
    );
    line(f, y + 17, "PgUp/PgDn прокрутить текст".to_string(), false);
}
fn session_lines(s: &Session) -> Vec<Line<'static>> {
    let e = &s.entries[s.index];
    let mut lines = vec![
        Line::from("По-русски"),
        Line::from(e.ru.clone()),
        Line::default(),
    ];
    lines.push(Line::from("Ваш ответ"));
    let spans = vec![Span::raw(format!("{}_", s.input))];
    lines.push(Line::from(spans));
    lines
}
fn result_lines(s: &Session) -> Vec<Line<'static>> {
    s.mistakes()
        .into_iter()
        .flat_map(|i| {
            [
                Line::from(format!("По-русски: {}", s.entries[i].ru)),
                Line::from(format!(
                    "Ваш ответ: {}",
                    if s.answers[i].is_empty() {
                        "(пустой ответ)"
                    } else {
                        &s.answers[i]
                    }
                )),
                Line::from(format!("Верный ответ: {}", s.entries[i].target)),
                Line::default(),
            ]
        })
        .collect()
}
fn wrapped(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    let mut out = vec![];
    let width = width.max(1);
    for line in lines {
        let mut spans = vec![];
        let mut used = 0;
        for span in line.spans {
            for g in course::graphemes(&span.content) {
                let cells = Line::from(g).width();
                if used + cells > width && !spans.is_empty() {
                    out.push(Line::from(std::mem::take(&mut spans)));
                    used = 0;
                }
                spans.push(Span::styled(g.to_string(), span.style));
                used += cells;
            }
        }
        out.push(Line::from(spans));
    }
    out
}
pub(crate) fn scroll_limit(app: &CourseApp, width: u16) -> usize {
    let width = width.saturating_sub(4) as usize;
    match &app.screen {
        Screen::Review(s) => wrapped(result_lines(s), width).len().saturating_sub(12),
        Screen::Session(s) => wrapped(session_lines(s), width).len().saturating_sub(10),
        Screen::Home | Screen::Result(_) | Screen::Training(_) | Screen::TrainingResult(_) => 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrapped_review_keeps_every_character_and_counts_long_answers() {
        let text = "очень длинный ответ ".repeat(100);
        let lines = wrapped(vec![Line::from(text.clone())], 20);
        assert!(lines.len() > 50);
        assert!(lines.iter().all(|l| l.width() <= 20));
        let restored = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect::<String>();
        assert_eq!(restored, text);
    }
}

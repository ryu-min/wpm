use crate::{
    domain::course::{self, Course, CourseApp, Mode, ResultAction, Screen, Session, Training},
    storage::course as storage,
    ui::{TypingWidget, course as view},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::time::Duration;

impl CourseApp {
    fn new(catalog: Course) -> Self {
        let (choice, notice) = storage::load();
        let mut app = Self {
            catalog,
            choice,
            field: 0,
            screen: Screen::Home,
            notice,
            save_failed: false,
            action: 0,
            scroll: 0,
        };
        app.reconcile();
        app
    }
    fn reconcile(&mut self) {
        if !self
            .catalog
            .modules
            .iter()
            .any(|m| m.id == self.choice.module)
        {
            self.choice.module = self.catalog.modules[0].id.clone();
        }
        for m in &self.catalog.modules {
            if !m
                .topics
                .iter()
                .any(|t| Some(&t.id) == self.choice.topics.get(&m.id))
            {
                self.choice
                    .topics
                    .insert(m.id.clone(), m.topics[0].id.clone());
            }
        }
    }
    pub fn module_index(&self) -> usize {
        self.catalog
            .modules
            .iter()
            .position(|m| m.id == self.choice.module)
            .unwrap_or(0)
    }
    pub fn topic(&self) -> &course::Topic {
        let m = &self.catalog.modules[self.module_index()];
        m.topics
            .iter()
            .find(|t| Some(&t.id) == self.choice.topics.get(&m.id))
            .unwrap_or(&m.topics[0])
    }
    fn save(&mut self) {
        match storage::save(&self.choice) {
            Ok(()) => {
                self.save_failed = false;
                self.notice = None;
            }
            Err(e) => {
                self.save_failed = true;
                self.notice = Some(format!("Ошибка сохранения ({e})"));
            }
        }
    }
    fn change(&mut self, step: isize) {
        match self.field {
            0 => {
                let n = (self.module_index() as isize + step)
                    .rem_euclid(self.catalog.modules.len() as isize)
                    as usize;
                self.choice.module = self.catalog.modules[n].id.clone();
            }
            1 => {
                let m = &self.catalog.modules[self.module_index()];
                let i = m
                    .topics
                    .iter()
                    .position(|t| t.id == self.topic().id)
                    .unwrap();
                let n = (i as isize + step).rem_euclid(m.topics.len() as isize) as usize;
                self.choice
                    .topics
                    .insert(m.id.clone(), m.topics[n].id.clone());
            }
            _ => {
                self.choice.mode = if self.choice.mode == Mode::Training {
                    Mode::Exam
                } else {
                    Mode::Training
                };
            }
        }
        self.save();
    }
    fn start(&mut self) {
        let t = self.topic();
        if t.entries.is_empty() {
            self.notice = Some(t.empty_reason().into());
            return;
        }
        self.screen = if self.choice.mode == Mode::Training {
            Screen::Training(Training::new(t.entries.clone(), t.title.clone()))
        } else {
            Screen::Session(Session::new(t.entries.clone(), Mode::Exam, t.title.clone()))
        };
        self.action = 0;
        self.scroll = 0;
    }
    fn finish_training(&mut self) {
        if let Screen::Training(t) = &mut self.screen {
            t.typing.update_stats();
            if t.typing.is_complete() {
                t.typing.finalize();
                let old = std::mem::replace(&mut self.screen, Screen::Home);
                if let Screen::Training(t) = old {
                    self.screen = Screen::TrainingResult(t);
                    self.action = 0;
                }
            }
        }
    }
    pub fn actions(&self, s: &Session) -> Vec<ResultAction> {
        let mut a = vec![];
        if s.mode == Mode::Exam {
            if !s.mistakes().is_empty() {
                a.push(ResultAction::ViewErrors);
                a.push(ResultAction::TrainErrors);
            }
            a.push(ResultAction::Repeat);
        } else {
            a.push(ResultAction::Repeat);
        }
        a.push(ResultAction::Home);
        a
    }
    fn key(&mut self, key: KeyCode) -> bool {
        if let Screen::TrainingResult(t) = &mut self.screen
            && TypingWidget::extend_result(&mut t.typing, KeyEvent::from(key))
        {
            return true;
        }
        if key == KeyCode::Esc {
            if matches!(self.screen, Screen::Home) {
                return false;
            }
            let old = std::mem::replace(&mut self.screen, Screen::Home);
            if let Screen::Review(s) = old {
                self.screen = Screen::Result(s);
            }
            self.scroll = 0;
            return true;
        }
        match &mut self.screen {
            Screen::Home => match key {
                KeyCode::Up => self.field = (self.field + 2) % 3,
                KeyCode::Down => self.field = (self.field + 1) % 3,
                KeyCode::Left => self.change(-1),
                KeyCode::Right => self.change(1),
                KeyCode::Enter => self.start(),
                KeyCode::Char('r' | 'R') if self.save_failed => self.save(),
                _ => {}
            },
            Screen::Training(t) => {
                TypingWidget::handle_input(&mut t.typing, KeyEvent::from(key));
                self.finish_training();
            }
            Screen::TrainingResult(_) => {
                if matches!(key, KeyCode::Up | KeyCode::Down) {
                    self.action = 1 - self.action;
                }
                if key == KeyCode::Enter {
                    let old = std::mem::replace(&mut self.screen, Screen::Home);
                    if self.action == 0
                        && let Screen::TrainingResult(t) = old
                    {
                        self.screen = Screen::Training(Training::new(t.entries, t.title));
                    }
                }
            }
            Screen::Session(s) => match key {
                KeyCode::Char(c) => {
                    s.input.push(c);
                    self.scroll = usize::MAX;
                }
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
                KeyCode::Backspace => {
                    course::backspace(&mut s.input);
                    self.scroll = usize::MAX;
                }
                KeyCode::Enter => {
                    s.answers.push(std::mem::take(&mut s.input));
                    s.index += 1;
                    self.scroll = 0;
                    if s.index == s.entries.len() {
                        let old = std::mem::replace(&mut self.screen, Screen::Home);
                        if let Screen::Session(done) = old {
                            self.screen = Screen::Result(done);
                            self.action = 0;
                            self.scroll = 0;
                        }
                    }
                }
                _ => {}
            },
            Screen::Review(_) => match key {
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
                _ => {}
            },
            Screen::Result(_) => {
                let old = std::mem::replace(&mut self.screen, Screen::Home);
                if let Screen::Result(s) = old {
                    let actions = self.actions(&s);
                    match key {
                        KeyCode::Up => {
                            self.action = (self.action + actions.len() - 1) % actions.len()
                        }
                        KeyCode::Down => self.action = (self.action + 1) % actions.len(),
                        KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
                        KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
                        KeyCode::Enter => {
                            let action = actions[self.action];
                            if action == ResultAction::Home {
                                return true;
                            }
                            if action == ResultAction::ViewErrors {
                                self.screen = Screen::Review(s);
                                self.scroll = 0;
                            } else if action == ResultAction::TrainErrors {
                                let entries =
                                    s.mistakes().iter().map(|&i| s.entries[i].clone()).collect();
                                self.screen = Screen::Training(Training::new(entries, s.title));
                            } else {
                                self.start();
                            }
                            return true;
                        }
                        _ => {}
                    }
                    self.screen = Screen::Result(s);
                }
            }
        }
        true
    }
}
pub fn run_spanish_course(mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
    let catalog = storage::catalog()?;
    if catalog.modules.is_empty() || catalog.modules.iter().any(|m| m.topics.is_empty()) {
        return Err(color_eyre::eyre::eyre!("Пустой каталог курса"));
    }
    let mut app = CourseApp::new(catalog);
    'running: loop {
        app.finish_training();
        let size = terminal.size()?;
        app.scroll = app.scroll.min(view::scroll_limit(&app, size.width));
        terminal.draw(|f| view::render(f, &app))?;
        if event::poll(Duration::from_millis(100))? {
            loop {
                let event = event::read()?;
                if matches!(event, Event::Resize(_, _)) && matches!(app.screen, Screen::Session(_))
                {
                    app.scroll = usize::MAX;
                }
                if let Event::Key(key) = event
                    && key.kind == KeyEventKind::Press
                {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                        || !app.key(key.code)
                    {
                        break 'running;
                    }
                }
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::course::{Choice, Entry};
    fn app() -> CourseApp {
        let catalog = Course {
            modules: vec![course::Module {
                id: "m1".into(),
                title: "¡Hola!".into(),
                topics: vec![course::Topic {
                    id: "t1".into(),
                    title: "Кафе".into(),
                    start: 1,
                    end: 1,
                    unavailable: false,
                    entries: vec![
                        Entry {
                            ru: "вода".into(),
                            target: "agua".into(),
                            alternatives: vec![],
                        },
                        Entry {
                            ru: "кофе".into(),
                            target: "café".into(),
                            alternatives: vec![],
                        },
                    ],
                }],
            }],
        };
        CourseApp {
            catalog,
            choice: Choice {
                module: "m1".into(),
                topics: std::collections::BTreeMap::from([("m1".into(), "t1".into())]),
                mode: Mode::Training,
            },
            field: 0,
            screen: Screen::Home,
            notice: None,
            save_failed: false,
            action: 0,
            scroll: 0,
        }
    }
    #[test]
    fn training_uses_translation_and_exam_keeps_answers() {
        let mut a = app();
        a.start();
        if let Screen::Training(t) = &a.screen {
            assert_eq!(t.typing.target_text(), "вода agua\nкофе café");
        } else {
            panic!();
        }
        for c in "вода agua".chars() {
            a.key(KeyCode::Char(c));
        }
        a.key(KeyCode::Enter);
        a.key(KeyCode::Backspace);
        if let Screen::Training(t) = &a.screen {
            assert_eq!(t.typing.get_input(), "вода agua");
        } else {
            panic!();
        }
        a.choice.mode = Mode::Exam;
        a.start();
        a.key(KeyCode::Enter);
        a.key(KeyCode::Enter);
        assert!(matches!(&a.screen,Screen::Result(s) if s.mistakes().len()==2));
        a.key(KeyCode::Down);
        a.key(KeyCode::Enter);
        assert!(matches!(&a.screen,Screen::Training(t) if t.entries.len()==2));
    }
}

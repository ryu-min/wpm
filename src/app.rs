use crate::{
    audio::Audio,
    configuration::Configuration,
    dictation::{Dictation, Phase},
    menu_widget::{MenuAction, MenuWidget},
    mode_select_widget::{ModeSelectAction, ModeSelectWidget, TestMode},
    result_widget::{ResultAction, ResultWidget},
    settings_widget::{SettingsAction, SettingsWidget},
    typing_widget::TypingWidget,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};
use std::time::Duration;
#[derive(Debug, PartialEq)]
enum Screen {
    Menu,
    ModeSelect,
    Settings,
    Typing,
    Result,
    Dictation,
    Error,
}
#[derive(Debug)]
pub struct App {
    running: bool,
    screen: Screen,
    menu_widget: MenuWidget,
    mode_select_widget: ModeSelectWidget,
    settings_widget: SettingsWidget,
    typing_widget: TypingWidget,
    result_widget: ResultWidget,
    config: Configuration,
    current_set: String,
    current_time: u32,
    current_mode: TestMode,
    dictation: Option<Dictation>,
    audio: Audio,
    error: String,
}
impl App {
    pub fn new() -> Self {
        let config = Configuration::new().expect("Failed to initialize configuration");
        let wordsets = config.get_wordset_names().expect("Failed to get wordsets");
        let translations = config
            .get_translation_set_names()
            .expect("Failed to get translation sets");
        let settings_widget =
            SettingsWidget::from_settings(wordsets.clone(), translations.clone(), &config.settings);
        Self {
            running: true,
            screen: Screen::Menu,
            menu_widget: MenuWidget::new(),
            mode_select_widget: ModeSelectWidget::new(wordsets, translations),
            settings_widget,
            typing_widget: TypingWidget::new(String::new()),
            result_widget: ResultWidget::new(),
            config,
            current_set: String::new(),
            current_time: 15,
            current_mode: TestMode::Typing,
            dictation: None,
            audio: Audio::default(),
            error: String::new(),
        }
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            self.poll_audio();
            if self.screen == Screen::Typing {
                self.typing_widget.update_stats();
                if self.typing_widget.is_complete() {
                    self.typing_widget.finalize();
                    self.result_widget.update(
                        self.typing_widget.get_wpm(),
                        self.typing_widget.get_accuracy(),
                        self.typing_widget.get_elapsed_time(),
                    );
                    self.screen = Screen::Result;
                }
            }
            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key_event(key);
                while self.running && event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()?
                        && key.kind == KeyEventKind::Press
                    {
                        self.on_key_event(key);
                    }
                }
            }
        }
        self.audio.shutdown();
        Ok(())
    }
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Menu => frame.render_widget(&self.menu_widget, area),
            Screen::ModeSelect => frame.render_widget(&self.mode_select_widget, area),
            Screen::Settings => frame.render_widget(&self.settings_widget, area),
            Screen::Typing => frame.render_widget(&self.typing_widget, area),
            Screen::Result => frame.render_widget(&self.result_widget, area),
            Screen::Dictation => {
                if let Some(session) = &self.dictation {
                    frame.render_widget(session, area);
                }
            }
            Screen::Error => frame.render_widget(
                Paragraph::new(format!(
                    "Vocabulary set unavailable or damaged. {}\nEsc: menu",
                    self.error
                )),
                area,
            ),
        }
    }
    fn start(&mut self, mode: TestMode, set: String, amount: u32) {
        self.audio.cancel();
        self.current_mode = mode;
        self.current_set = set.clone();
        self.current_time = amount;
        match mode {
            TestMode::Typing => match self.config.get_shuffled_words(&set) {
                Ok(words) if !words.is_empty() => {
                    self.typing_widget = TypingWidget::new(words.join(" "))
                        .with_time_limit(amount as u64)
                        .with_layout_conversion(self.config.settings.typing_layout_conversion);
                    self.screen = Screen::Typing;
                }
                _ => self.set_error(),
            },
            TestMode::Translation => match self.config.get_shuffled_vocabulary(&set) {
                Ok(set) => {
                    let (text, policies) = set.translation();
                    self.typing_widget = TypingWidget::new(text)
                        .with_time_limit(amount as u64)
                        .with_layout_conversion(true)
                        .with_languages(policies);
                    self.screen = Screen::Typing;
                }
                Err(_) => self.set_error(),
            },
            TestMode::Dictation => match self.config.get_shuffled_vocabulary(&set) {
                Ok(set) if set.target_language == "es" => {
                    match Dictation::new(set.entries, amount as usize).map(|session| {
                        session.with_case_insensitive(set.id == "ru_es_cafe_beginner")
                    }) {
                        Ok(mut session) => {
                            session.operation = self.audio.check();
                            self.dictation = Some(session);
                            self.screen = Screen::Dictation;
                        }
                        Err(_) => self.set_error(),
                    }
                }
                _ => self.set_error(),
            },
        }
    }
    fn set_error(&mut self) {
        self.error = "Select another dataset in Settings or Select Mode.".into();
        self.screen = Screen::Error;
    }
    fn repeat_audio(&mut self) {
        if let Some(session) = &mut self.dictation {
            session.operation = self.audio.speak(session.current().es.clone());
            session.status = "Playing…".into();
        }
    }
    fn poll_audio(&mut self) {
        while let Some(event) = self.audio.poll() {
            if self.screen != Screen::Dictation {
                continue;
            }
            let Some(session) = &mut self.dictation else {
                continue;
            };
            if event.operation != session.operation {
                continue;
            }
            if session.audio_event(
                event.operation,
                event.result.map_err(|error| error.message().into()),
            ) {
                self.repeat_audio();
            }
        }
    }
    fn menu(&mut self) {
        self.audio.cancel();
        self.dictation = None;
        self.screen = Screen::Menu;
    }
    fn on_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.audio.cancel();
            self.running = false;
            return;
        }
        match self.screen {
            Screen::Menu => {
                if let Some(action) = self.menu_widget.handle_input(key) {
                    match action {
                        MenuAction::QuickStart => {
                            let s = &self.config.settings;
                            let (mode, set, amount) = match s.quick_start_mode.as_str() {
                                "translation" => (
                                    TestMode::Translation,
                                    s.quick_start_translation_set.clone(),
                                    s.quick_start_time,
                                ),
                                "dictation" => (
                                    TestMode::Dictation,
                                    s.quick_start_dictation_set.clone(),
                                    s.quick_start_dictation_length,
                                ),
                                _ => (
                                    TestMode::Typing,
                                    s.quick_start_wordset.clone(),
                                    s.quick_start_time,
                                ),
                            };
                            self.start(mode, set, amount);
                        }
                        MenuAction::Translation => self.start(
                            TestMode::Translation,
                            self.config.settings.translation_set.clone(),
                            self.config.settings.translation_time,
                        ),
                        MenuAction::Dictation => self.start(
                            TestMode::Dictation,
                            self.config.settings.dictation_set.clone(),
                            self.config.settings.dictation_length,
                        ),
                        MenuAction::SelectMode => {
                            self.mode_select_widget.reset();
                            self.mode_select_widget.set_defaults(&self.config.settings);
                            self.screen = Screen::ModeSelect;
                        }
                        MenuAction::Settings => {
                            self.settings_widget = SettingsWidget::from_settings(
                                self.config.get_wordset_names().unwrap_or_default(),
                                self.config.get_translation_set_names().unwrap_or_default(),
                                &self.config.settings,
                            );
                            self.screen = Screen::Settings;
                        }
                        MenuAction::Exit => self.running = false,
                    }
                }
            }
            Screen::ModeSelect => {
                if let Some(action) = self.mode_select_widget.handle_input(key) {
                    match action {
                        ModeSelectAction::Exit => self.menu(),
                        ModeSelectAction::Start => {
                            let mode = self.mode_select_widget.selected_mode();
                            let (set, amount) = match mode {
                                TestMode::Typing => (
                                    self.mode_select_widget.selected_wordset().into(),
                                    self.mode_select_widget.selected_time(),
                                ),
                                TestMode::Translation => (
                                    self.mode_select_widget.selected_translation_set().into(),
                                    self.mode_select_widget.selected_time(),
                                ),
                                TestMode::Dictation => (
                                    self.mode_select_widget.selected_dictation_set().into(),
                                    self.mode_select_widget.selected_length(),
                                ),
                            };
                            self.start(mode, set, amount);
                        }
                    }
                }
            }
            Screen::Settings => {
                if let Some(action) = self.settings_widget.handle_input(key) {
                    match action {
                        SettingsAction::Save => {
                            self.config.settings = self.settings_widget.draft.clone();
                            if self.config.save_settings().is_err() {
                                self.error = "Unable to save settings. Esc: menu".into();
                                self.screen = Screen::Error;
                            } else {
                                self.menu();
                            }
                        }
                        SettingsAction::Exit => self.menu(),
                    }
                }
            }
            Screen::Typing => {
                if self.typing_widget.handle_input(key).is_some() {
                    self.typing_widget.reset();
                    self.menu();
                }
            }
            Screen::Result => {
                if self.typing_widget.extend_result(key) {
                    self.result_widget.update(
                        self.typing_widget.get_wpm(),
                        self.typing_widget.get_accuracy(),
                        self.typing_widget.get_elapsed_time(),
                    );
                    return;
                }
                if let Some(action) = self.result_widget.handle_input(key) {
                    match action {
                        ResultAction::Restart => self.start(
                            self.current_mode,
                            self.current_set.clone(),
                            self.current_time,
                        ),
                        ResultAction::Menu => self.menu(),
                    }
                }
            }
            Screen::Error => {
                if key.code == KeyCode::Esc {
                    self.menu();
                }
            }
            Screen::Dictation => {
                if key.code == KeyCode::Esc {
                    self.menu();
                    return;
                }
                let phase = self.dictation.as_ref().unwrap().phase;
                match (phase, key.code) {
                    (Phase::Finished, KeyCode::Enter) => self.start(
                        TestMode::Dictation,
                        self.current_set.clone(),
                        self.current_time,
                    ),
                    (Phase::Input | Phase::Checked, KeyCode::F(2)) => self.repeat_audio(),
                    (Phase::Input, KeyCode::Char(ch)) => self.dictation.as_mut().unwrap().push(ch),
                    (Phase::Input, KeyCode::Backspace) => {
                        self.dictation.as_mut().unwrap().backspace()
                    }
                    (Phase::Input, KeyCode::Enter) if self.dictation.as_mut().unwrap().submit() => {
                        self.audio.cancel();
                    }
                    (Phase::Checked, KeyCode::Enter) => {
                        self.audio.cancel();
                        if self.dictation.as_mut().unwrap().advance() {
                            self.repeat_audio();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

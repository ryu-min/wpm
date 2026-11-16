use crate::{
    domain::{TestMode, TypingSession},
    storage::Configuration,
    ui::{
        MenuAction, MenuWidget, ModeSelectAction, ModeSelectWidget, ResultAction, ResultWidget,
        SettingsAction, SettingsWidget, TypingWidget,
    },
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
    Error,
}
#[derive(Debug)]
pub(crate) struct App {
    running: bool,
    screen: Screen,
    menu_widget: MenuWidget,
    mode_select_widget: ModeSelectWidget,
    settings_widget: SettingsWidget,
    typing_session: TypingSession,
    result_widget: ResultWidget,
    config: Configuration,
    current_set: String,
    current_time: u32,
    current_mode: TestMode,
    error: String,
}
impl App {
    pub(crate) fn new() -> Self {
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
            typing_session: TypingSession::new(String::new()),
            result_widget: ResultWidget::new(),
            config,
            current_set: String::new(),
            current_time: 15,
            current_mode: TestMode::Typing,
            error: String::new(),
        }
    }
    pub(crate) fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            if self.screen == Screen::Typing {
                self.typing_session.update_stats();
                if self.typing_session.is_complete() {
                    self.typing_session.finalize();
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
        Ok(())
    }
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match self.screen {
            Screen::Menu => frame.render_widget(&self.menu_widget, area),
            Screen::ModeSelect => frame.render_widget(&self.mode_select_widget, area),
            Screen::Settings => frame.render_widget(&self.settings_widget, area),
            Screen::Typing => frame.render_widget(TypingWidget::new(&self.typing_session), area),
            Screen::Result => {
                frame.render_widget(self.result_widget.view(&self.typing_session), area)
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
        self.current_mode = mode;
        self.current_set = set.clone();
        self.current_time = amount;
        match mode {
            TestMode::Typing => match self.config.get_shuffled_words(&set) {
                Ok(words) if !words.is_empty() => {
                    self.typing_session = TypingSession::new(words.join(" "))
                        .with_time_limit(amount as u64)
                        .with_layout_conversion(self.config.settings.typing_layout_conversion);
                    self.screen = Screen::Typing;
                }
                _ => self.set_error(),
            },
            TestMode::Translation => match self.config.get_shuffled_vocabulary(&set) {
                Ok(set) => {
                    let (text, policies) = set.translation();
                    self.typing_session = TypingSession::new(text)
                        .with_time_limit(amount as u64)
                        .with_layout_conversion(true)
                        .with_languages(policies);
                    self.screen = Screen::Typing;
                }
                Err(_) => self.set_error(),
            },
        }
    }
    fn set_error(&mut self) {
        self.error = "Select another dataset in Settings or Select Mode.".into();
        self.screen = Screen::Error;
    }
    fn menu(&mut self) {
        self.screen = Screen::Menu;
    }
    fn on_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
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
                if TypingWidget::handle_input(&mut self.typing_session, key).is_some() {
                    self.typing_session.reset();
                    self.menu();
                }
            }
            Screen::Result => {
                if TypingWidget::extend_result(&mut self.typing_session, key) {
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
        }
    }
}

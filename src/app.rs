use crate::configuration::Configuration;
use crate::menu_widget::MenuWidget;
use crate::mode_select_widget::{ModeSelectWidget, TestMode};
use crate::result_widget::ResultWidget;
use crate::settings_widget::SettingsWidget;
use crate::typing_widget::TypingWidget;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

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
    current_wordset: Option<String>,
    current_translation_set: Option<String>,
    current_time: Option<u32>,
    current_mode: TestMode,
}

#[derive(Debug, PartialEq)]
enum Screen {
    Menu,
    ModeSelect,
    Settings,
    Typing,
    Result,
}

impl App {
    pub fn new() -> Self {
        let config = Configuration::new().expect("Failed to initialize configuration");
        let wordset_names = config.get_wordset_names().expect("Failed to get wordsets");
        let translation_set_names = config
            .get_translation_set_names()
            .unwrap_or_else(|_| vec!["ru_en_basic".to_string()]);
        let settings = &config.settings;

        let settings_widget = SettingsWidget::new(
            wordset_names.clone(),
            translation_set_names.clone(),
            &settings.quick_start_mode,
            &settings.quick_start_wordset,
            &settings.quick_start_translation_set,
            settings.quick_start_time,
            &settings.translation_set,
            settings.translation_time,
        );

        Self {
            running: true,
            screen: Screen::Menu,
            menu_widget: MenuWidget::new(),
            mode_select_widget: ModeSelectWidget::new(wordset_names, translation_set_names),
            settings_widget,
            typing_widget: TypingWidget::new(String::new()),
            result_widget: ResultWidget::new(),
            config,
            current_wordset: None,
            current_translation_set: None,
            current_time: None,
            current_mode: TestMode::Typing,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            if self.screen == Screen::Typing {
                self.typing_widget.update_stats();
                if self.typing_widget.is_complete() {
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
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Menu => frame.render_widget(&self.menu_widget, frame.area()),
            Screen::ModeSelect => frame.render_widget(&self.mode_select_widget, frame.area()),
            Screen::Settings => frame.render_widget(&self.settings_widget, frame.area()),
            Screen::Typing => frame.render_widget(&self.typing_widget, frame.area()),
            Screen::Result => frame.render_widget(&self.result_widget, frame.area()),
        }
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('c')
            && key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            self.quit();
            return;
        }

        match self.screen {
            Screen::Menu => {
                if let Some(action) = self.menu_widget.handle_input(key) {
                    match action {
                        crate::menu_widget::MenuAction::QuickStart => {
                            let time = self.config.settings.quick_start_time;
                            if self.config.settings.quick_start_mode == "translation" {
                                let set_name =
                                    self.config.settings.quick_start_translation_set.clone();
                                if let Ok(pairs) =
                                    self.config.get_shuffled_translation_pairs(&set_name)
                                {
                                    let text = build_translation_target(&pairs);
                                    self.typing_widget =
                                        TypingWidget::new(text).with_time_limit(time as u64);
                                    self.current_mode = TestMode::Translation;
                                    self.current_wordset = None;
                                    self.current_translation_set = Some(set_name);
                                    self.current_time = Some(time);
                                    self.screen = Screen::Typing;
                                }
                            } else if let Ok(words) = self.config.quick_start_words() {
                                let text = words.join(" ");
                                self.typing_widget =
                                    TypingWidget::new(text).with_time_limit(time as u64);
                                self.current_mode = TestMode::Typing;
                                self.current_wordset =
                                    Some(self.config.settings.quick_start_wordset.clone());
                                self.current_translation_set = None;
                                self.current_time = Some(time);
                                self.screen = Screen::Typing;
                            }
                        }
                        crate::menu_widget::MenuAction::Translation => {
                            let time = self.config.settings.translation_time;
                            let preferred_set = self.config.settings.translation_set.clone();
                            let set_name =
                                if self.config.get_translation_pairs(&preferred_set).is_ok() {
                                    preferred_set
                                } else {
                                    self.config
                                        .get_translation_set_names()
                                        .ok()
                                        .and_then(|v| v.into_iter().next())
                                        .unwrap_or_else(|| "ru_en_a1".to_string())
                                };
                            if let Ok(pairs) = self.config.get_shuffled_translation_pairs(&set_name)
                            {
                                let text = build_translation_target(&pairs);
                                self.typing_widget =
                                    TypingWidget::new(text).with_time_limit(time as u64);
                                self.current_mode = TestMode::Translation;
                                self.current_wordset = None;
                                self.current_translation_set = Some(set_name);
                                self.current_time = Some(time);
                                self.screen = Screen::Typing;
                            }
                        }
                        crate::menu_widget::MenuAction::SelectMode => {
                            self.mode_select_widget.reset();
                            self.screen = Screen::ModeSelect;
                        }
                        crate::menu_widget::MenuAction::Settings => {
                            let settings = &self.config.settings;
                            self.settings_widget = SettingsWidget::new(
                                self.config.get_wordset_names().unwrap_or_default(),
                                self.config.get_translation_set_names().unwrap_or_default(),
                                &settings.quick_start_mode,
                                &settings.quick_start_wordset,
                                &settings.quick_start_translation_set,
                                settings.quick_start_time,
                                &settings.translation_set,
                                settings.translation_time,
                            );
                            self.screen = Screen::Settings;
                        }
                        crate::menu_widget::MenuAction::Exit => {
                            self.quit();
                        }
                    }
                }
            }
            Screen::ModeSelect => {
                if let Some(action) = self.mode_select_widget.handle_input(key) {
                    match action {
                        crate::mode_select_widget::ModeSelectAction::Start => {
                            let time = self.mode_select_widget.selected_time();
                            let mode = self.mode_select_widget.selected_mode();
                            self.current_mode = mode;
                            match mode {
                                TestMode::Typing => {
                                    let wordset =
                                        self.mode_select_widget.selected_wordset().to_string();
                                    if let Ok(words) = self.config.get_shuffled_words(&wordset) {
                                        let text = words.join(" ");
                                        self.typing_widget =
                                            TypingWidget::new(text).with_time_limit(time as u64);
                                        self.current_wordset = Some(wordset);
                                        self.current_translation_set = None;
                                        self.current_time = Some(time);
                                        self.screen = Screen::Typing;
                                    }
                                }
                                TestMode::Translation => {
                                    let set_name = self
                                        .mode_select_widget
                                        .selected_translation_set()
                                        .to_string();
                                    if let Ok(pairs) =
                                        self.config.get_shuffled_translation_pairs(&set_name)
                                    {
                                        let text = build_translation_target(&pairs);
                                        self.typing_widget =
                                            TypingWidget::new(text).with_time_limit(time as u64);
                                        self.current_wordset = None;
                                        self.current_translation_set = Some(set_name);
                                        self.current_time = Some(time);
                                        self.screen = Screen::Typing;
                                    }
                                }
                            }
                        }
                        crate::mode_select_widget::ModeSelectAction::Exit => {
                            self.screen = Screen::Menu;
                        }
                    }
                }
            }
            Screen::Settings => {
                if let Some(action) = self.settings_widget.handle_input(key) {
                    match action {
                        crate::settings_widget::SettingsAction::Save => {
                            self.config.settings.quick_start_mode =
                                self.settings_widget.quick_start_mode().to_string();
                            self.config.settings.quick_start_wordset =
                                self.settings_widget.quick_start_wordset().to_string();
                            self.config.settings.quick_start_translation_set = self
                                .settings_widget
                                .quick_start_translation_set()
                                .to_string();
                            self.config.settings.quick_start_time =
                                self.settings_widget.quick_start_time();
                            self.config.settings.translation_set =
                                self.settings_widget.translation_set().to_string();
                            self.config.settings.translation_time =
                                self.settings_widget.translation_time();
                            self.config.save_settings().ok();
                            self.screen = Screen::Menu;
                        }
                        crate::settings_widget::SettingsAction::Exit => {
                            self.screen = Screen::Menu;
                        }
                        _ => {}
                    }
                }
            }
            Screen::Typing => {
                if let Some(action) = self.typing_widget.handle_input(key) {
                    match action {
                        crate::typing_widget::TypingAction::Exit => {
                            self.screen = Screen::Menu;
                            self.typing_widget.reset();
                        }
                    }
                }
            }
            Screen::Result => {
                if let Some(action) = self.result_widget.handle_input(key) {
                    match action {
                        crate::result_widget::ResultAction::Restart => {
                            if let Some(time) = self.current_time {
                                match self.current_mode {
                                    TestMode::Typing => {
                                        if let Some(wordset) = &self.current_wordset
                                            && let Ok(words) =
                                                self.config.get_shuffled_words(wordset)
                                        {
                                            let text = words.join(" ");
                                            self.typing_widget = TypingWidget::new(text)
                                                .with_time_limit(time as u64);
                                        }
                                    }
                                    TestMode::Translation => {
                                        if let Some(set_name) = &self.current_translation_set
                                            && let Ok(pairs) =
                                                self.config.get_shuffled_translation_pairs(set_name)
                                        {
                                            let text = build_translation_target(&pairs);
                                            self.typing_widget = TypingWidget::new(text)
                                                .with_time_limit(time as u64);
                                        }
                                    }
                                }
                            }
                            self.screen = Screen::Typing;
                        }
                        crate::result_widget::ResultAction::Menu => {
                            self.screen = Screen::Menu;
                            self.typing_widget.reset();
                        }
                    }
                }
            }
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

fn build_translation_target(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(ru, en)| format!("{ru} {en}"))
        .collect::<Vec<String>>()
        .join("\n")
}

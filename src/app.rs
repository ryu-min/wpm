use crate::menu_widget::MenuWidget;
use crate::mode_select_widget::ModeSelectWidget;
use crate::result_widget::ResultWidget;
use crate::typing_widget::TypingWidget;
use crate::wordset::WordsetDb;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use std::time::Duration;

#[derive(Debug)]
pub struct App {
    running: bool,
    screen: Screen,
    menu_widget: MenuWidget,
    mode_select_widget: ModeSelectWidget,
    typing_widget: TypingWidget,
    result_widget: ResultWidget,
    wordset_db: WordsetDb,
}

#[derive(Debug, PartialEq)]
enum Screen {
    Menu,
    ModeSelect,
    Typing,
    Result,
}

impl App {
    pub fn new() -> Self {
        let wordset_db = WordsetDb::new().expect("Failed to initialize database");
        let wordset_names = wordset_db.get_wordset_names().expect("Failed to get wordsets");

        Self {
            running: true,
            screen: Screen::Menu,
            menu_widget: MenuWidget::new(),
            mode_select_widget: ModeSelectWidget::new(wordset_names),
            typing_widget: TypingWidget::new(String::new()),
            result_widget: ResultWidget::new(),
            wordset_db,
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
            
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.on_key_event(key);
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Menu => frame.render_widget(&self.menu_widget, frame.area()),
            Screen::ModeSelect => frame.render_widget(&self.mode_select_widget, frame.area()),
            Screen::Typing => frame.render_widget(&self.typing_widget, frame.area()),
            Screen::Result => frame.render_widget(&self.result_widget, frame.area()),
        }
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            match self.screen {
                Screen::Menu => self.quit(),
                _ => {
                    self.screen = Screen::Menu;
                    self.typing_widget.reset();
                }
            }
            return;
        }
        
        if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
            self.quit();
            return;
        }

        match self.screen {
            Screen::Menu => {
                match self.menu_widget.handle_input(key) {
                    crate::menu_widget::MenuAction::QuickStart => {
                        if let Ok(words) = self.wordset_db.quick_start_words() {
                            let text = words.join(" ");
                            self.typing_widget = TypingWidget::new(text).with_time_limit(15);
                            self.screen = Screen::Typing;
                        }
                    }
                    crate::menu_widget::MenuAction::SelectMode => {
                        self.mode_select_widget.reset();
                        self.screen = Screen::ModeSelect;
                    }
                    crate::menu_widget::MenuAction::Exit => {
                        self.quit();
                    }
                    _ => {}
                }
            }
            Screen::ModeSelect => {
                match self.mode_select_widget.handle_input(key) {
                    crate::mode_select_widget::ModeSelectAction::Start => {
                        let wordset = self.mode_select_widget.selected_wordset().to_string();
                        let time = self.mode_select_widget.selected_time();
                        if let Ok(words) = self.wordset_db.get_shuffled_words(&wordset) {
                            let text = words.join(" ");
                            self.typing_widget = TypingWidget::new(text).with_time_limit(time as u64);
                            self.screen = Screen::Typing;
                        }
                    }
                    _ => {}
                }
            }
            Screen::Typing => {
                self.typing_widget.handle_input(key);
            }
            Screen::Result => {
                match self.result_widget.handle_input(key) {
                    crate::result_widget::ResultAction::Restart => {
                        self.typing_widget.reset();
                        self.screen = Screen::Typing;
                    }
                    crate::result_widget::ResultAction::Menu => {
                        self.screen = Screen::Menu;
                        self.typing_widget.reset();
                    }
                    _ => {}
                }
            }
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}
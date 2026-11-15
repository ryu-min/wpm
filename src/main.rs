mod app;
mod audio;
mod configuration;
mod dataset;
mod dictation;
mod dictation_widget;
mod language_input;
mod menu_widget;
mod mode_select_widget;
mod result_widget;
mod settings_widget;
mod typing_widget;
mod vocabulary;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}

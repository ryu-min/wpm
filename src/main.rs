mod app;
mod configuration;
mod menu_widget;
mod mode_select_widget;
mod result_widget;
mod settings_widget;
mod typing_widget;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}
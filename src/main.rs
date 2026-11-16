mod app;
mod dataset;
mod domain;
mod storage;
#[cfg(test)]
mod tests;
mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}

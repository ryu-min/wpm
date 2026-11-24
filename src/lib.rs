mod app;
mod dataset;
mod domain;
mod storage;
#[cfg(test)]
mod tests;
mod ui;

/// Run the typing and translation application.
pub fn run() -> color_eyre::Result<()> {
    with_terminal(|terminal| app::App::new().run(terminal))
}

/// Run the offline Spanish course.
pub fn run_spanish_course() -> color_eyre::Result<()> {
    with_terminal(app::run_spanish_course)
}

fn with_terminal(
    run: impl FnOnce(ratatui::DefaultTerminal) -> color_eyre::Result<()>,
) -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

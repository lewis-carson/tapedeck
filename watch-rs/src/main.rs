pub use app::App;

pub mod app;

use std::io;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let stdin = io::stdin(); 
    let reader = stdin.lock();

    let result = App::new(reader).run(terminal);
    ratatui::restore();

    result
}
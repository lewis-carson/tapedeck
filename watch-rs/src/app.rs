use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use std::io::{self, BufRead};

#[derive(Debug, Default)]
pub struct App<I> {
    /// Is the application running?
    running: bool,
    /// Iterator for stdin input
    input: I,
}

impl<I> App<I>
where
    I: Iterator<Item = String>,
{
    /// Construct a new instance of [`App`].
    pub fn new(input: I) -> Self {
        Self {
            running: false,
            input,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_input()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    /// Reads the stdin input and updates the state of [`App`].
    fn handle_input(&mut self) -> Result<()> {
        if let Some(line) = self.input.next() {
            self.on_input(line);
        }
        Ok(())
    }

    /// Handles the input and updates the state of [`App`].
    fn on_input(&mut self, input: String) {
        match input.trim() {
            "q" | "Q" | "exit" | "quit" => self.quit(),
            // Add other input handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
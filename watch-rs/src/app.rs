use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use datatypes::world_builder::World;
use ratatui::{
    layout::{Constraint, Direction, Layout}, style::Stylize, symbols::line, text::Line, widgets::{Block, Clear, Paragraph}, DefaultTerminal, Frame
};
use std::{
    collections::HashMap, fs::File, io::{self, BufRead, BufReader}, sync::mpsc::channel, thread, time::Duration
};


/*
let (tx, rx) = channel();

// spawn a thread to read from /dev/stdin
// entering raw mode in crossterm seems to break std::io::stdin
thread::spawn(move || {
    let file = File::open("/dev/stdin").unwrap();
    for line in BufReader::new(file).lines() {
        tx.send(line.unwrap()).ok();
    }
}); */

#[derive(Debug)]
enum StreamObject {
    Event(datatypes::Event),
    World(datatypes::world_builder::World),
}

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    streams: HashMap<String, Vec<StreamObject>>,
    streams_channels: HashMap<String, std::sync::mpsc::Receiver<StreamObject>>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    fn new_stream(&mut self, name: &str, handle: impl Fn(&std::sync::mpsc::Sender<StreamObject>) + Send + 'static) {
        let (tx, rx) = channel();
        self.streams.insert(name.to_string(), Vec::new());
        self.streams_channels.insert(name.to_string(), rx);

        thread::spawn(move || {
            handle(&tx);
        });
    }

    fn launch_command(command: &str) -> impl Iterator<Item = String> {
        let child = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let reader = BufReader::new(child.stdout.unwrap());
        reader.lines().map(|line| line.unwrap())
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        terminal.clear();
        
        self.new_stream("fulls", |tx| {
            // launch a command and read from its stdout
            let command = "tail -fq data/*";
            for line in Self::launch_command(command) {
                let line: datatypes::Event = serde_json::from_str(&line).unwrap();

                if let datatypes::EventType::FullOrderBook(_) = line.event {
                    tx.send(StreamObject::Event(line)).ok();
                }
                
            }
        });

        self.new_stream("partials", |tx| {
            // launch a command and read from its stdout
            let command = "tail -fq data/*";
            for line in Self::launch_command(command) {
                let line: datatypes::Event = serde_json::from_str(&line).unwrap();

                if let datatypes::EventType::PartialOrderBook(_) = line.event {
                    tx.send(StreamObject::Event(line)).ok();
                }
            }
        });


        self.new_stream("world", |tx| {
            // launch a command and read from its stdout
            let command = "tail -fq data/*";

            let world_builder = datatypes::world_builder::WorldBuilder::new(Box::new(Self::launch_command(command).map(|line| {
                let event: datatypes::Event = serde_json::from_str(&line).unwrap();
                Ok(event)
            })));

            for world in world_builder {
                tx.send(StreamObject::World(world.unwrap())).ok();
            }
        });

        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/master/examples>
    fn draw(&mut self, frame: &mut Frame) {
        for (name, rx) in self.streams_channels.iter() {
            if let Ok(line) = rx.try_recv() {
                self.streams.get_mut(name).unwrap().push(line);
            }
        }

        let master_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let middle_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(1)
            ])
            .split(master_layout[1]);

        let middle_right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(middle_layout[2]);

        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(master_layout[2]);

        let header_area = master_layout[0];
        let world_area = middle_layout[0];
        let graph_area = middle_layout[1];
        let orders_area = middle_right_layout[0];
        let fills_area = middle_right_layout[1];
        let fulls_area = bottom_layout[0];
        let partials_area = bottom_layout[1];

        frame.render_widget(
            Paragraph::new("top")
                .block(Block::bordered().title("top"))
                .centered(),
            header_area,
        );

        // if world stream has some length, destructure it to get the last element
        let world = self.streams.get("world").unwrap().last().map(|world| {
            if let StreamObject::World(world) = world {
                // get vec of order books
                let mut obs = world.order_books.keys().map(|s| s.clone()).collect::<Vec<String>>();
                obs.sort();

                obs.join("\n")
            } else {
                "".to_string()
            }
        }).unwrap_or("".to_string());

        frame.render_widget(
            Paragraph::new(world)
                .block(Block::bordered().title("World")),
            world_area,
        );

        frame.render_widget(
            Paragraph::new("middle")
                .block(Block::bordered().title("middle"))
                .centered(),
            graph_area,
        );

        frame.render_widget(
            Paragraph::new("right")
                .block(Block::bordered().title("right"))
                .centered(),
            orders_area,
        );

        frame.render_widget(
            Paragraph::new("right")
                .block(Block::bordered().title("right"))
                .centered(),
            fills_area,
        );

        // map all of fulls to FullOrderBook vector
        let fulls = self.streams.get("fulls").unwrap().iter().filter_map(|stream| {
            if let StreamObject::Event(event) = stream {
                let receive_time = (event.receive_time / 1000) as i64;
                let nanos = ((event.receive_time % 1000) * 1000000) as u32;
                let time = chrono::DateTime::from_timestamp(receive_time, nanos).unwrap().format("%H:%M:%S%.3f").to_string();
                Some(format!("{} | {}", time, event.symbol))
            } else {
                None
            }
        }).collect::<Vec<_>>().join("\n");

        let fulls_offset = self.streams.get("fulls").unwrap().len().saturating_sub(fulls_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(fulls)
                .scroll((fulls_offset as u16, 0))
                .block(Block::bordered().title("Full Books")),
            fulls_area,
        );


        let partials = self.streams.get("partials").unwrap().iter().filter_map(|stream| {
            if let StreamObject::Event(event) = stream {
                let receive_time = (event.receive_time / 1000) as i64;
                let nanos = ((event.receive_time % 1000) * 1000000) as u32;
                let time = chrono::DateTime::from_timestamp(receive_time, nanos).unwrap().format("%H:%M:%S%.3f").to_string();
                Some(format!("{} | {}", time, event.symbol))
            } else {
                None
            }
        }).collect::<Vec<_>>().join("\n");

        let partials_offset = self.streams.get("partials").unwrap().len().saturating_sub(partials_area.height as usize - 2);
        frame.render_widget(
            Paragraph::new(partials)
                .scroll((partials_offset as u16, 0))
                .block(Block::bordered().title("Partial Books")),
            partials_area,
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

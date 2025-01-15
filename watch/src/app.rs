use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use datatypes::{EventType, world_builder::World};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::line,
    text::Line,
    widgets::{Block, Clear, Paragraph},
};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    sync::mpsc::channel,
    thread,
    time::Duration,
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

    fn new_stream(
        &mut self,
        name: &str,
        handle: impl Fn(&std::sync::mpsc::Sender<StreamObject>) + Send + 'static,
    ) {
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

        self.new_stream("open_orders", |tx| {
            // launch a command and read from its stdout
            let command = "tail -fq data/*";
            for line in Self::launch_command(command) {
                let line: datatypes::Event = serde_json::from_str(&line).unwrap();

                if let datatypes::EventType::OrderTradeEvent(_) = line.event {
                    tx.send(StreamObject::Event(line)).ok();
                }
            }
        });

        self.new_stream("world", |tx| {
            // launch a command and read from its stdout
            let command = "tail -fq data/*";

            let world_builder = datatypes::world_builder::WorldBuilder::new(Box::new(
                Self::launch_command(command).map(|line| {
                    let event: datatypes::Event = serde_json::from_str(&line).unwrap();
                    Ok(event)
                }),
            ));

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

    fn timestamp_to_string(timestamp: u64) -> String {
        let receive_time = (timestamp / 1000) as i64;
        let nanos = ((timestamp % 1000) * 1000000) as u32;
        chrono::DateTime::from_timestamp(receive_time, nanos)
            .unwrap()
            .format("%H:%M:%S%.3f")
            .to_string()
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

        // cut streams down to 100 elements
        for (_, stream) in self.streams.iter_mut() {
            if stream.len() > 100 {
                stream.drain(0..stream.len() - 100);
            }
        }

        let master_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .split(frame.area());

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(3),
            ])
            .split(master_layout[1]);

        let main_right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(main_layout[1]);

        let main_right_top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
            ])
            .split(main_right[0]);

        let main_right_bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
            ])
            .split(main_right[1]);


        let header_area = master_layout[0];
        let world_area = main_layout[0];

        let orders_area = main_right_top[0];
        let fills_area = main_right_top[1];

        let fulls_area = main_right_bottom[0];
        let partials_area = main_right_bottom[1];

        frame.render_widget(
            Paragraph::new("top")
                .block(Block::bordered().title("top"))
                .centered(),
            header_area,
        );

        // if world stream has some length, destructure it to get the last element
        let world = self
            .streams
            .get("world")
            .unwrap()
            .last()
            .map(|world| {
                if let StreamObject::World(world) = world {
                    world.account_information.clone().map(|account| {
                        account
                            .balances
                            .into_iter()
                            .map(|balance| format!("{}: {}", balance.asset, balance.free))
                            .collect::<Vec<_>>()
                            .join("\n")
                    }).unwrap_or("".to_string())
                } else {
                    "".to_string()
                }
            })
            .unwrap_or("".to_string());

        frame.render_widget(
            Paragraph::new(world).block(Block::bordered().title("World")),
            world_area,
        );


        let open_orders = self
            .streams
            .get("open_orders")
            .unwrap()
            .iter()
            .filter_map(|stream| {
                let event = if let StreamObject::Event(event) = stream {
                    event
                } else {
                    panic!("Expected Event")
                };

                if let EventType::OrderTradeEvent(order) = &event.event {
                    let time = Self::timestamp_to_string(event.receive_time);
                    Some(format!(
                        "{} | {} | {} | {} | {}",
                        time,
                        order.symbol,
                        order.side,
                        order.price,
                        order.qty,
                    ))
                } else {
                    panic!("Expected OrderTradeEvent")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let open_orders_offset = self
            .streams
            .get("open_orders")
            .unwrap()
            .len()
            .saturating_sub(orders_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(open_orders)
                .block(Block::bordered().title("Orders"))
                .scroll((open_orders_offset as u16, 0)),
            orders_area,
        );

        frame.render_widget(
            Paragraph::new("")
                .block(Block::bordered().title("Fills"))
                .centered(),
            fills_area,
        );

        // map all of fulls to FullOrderBook vector
        let fulls = self
            .streams
            .get("fulls")
            .unwrap()
            .iter()
            .filter_map(|stream| {
                if let StreamObject::Event(event) = stream {
                    let time = Self::timestamp_to_string(event.receive_time);
                    Some(format!("{} | {}", time, event.symbol))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let fulls_offset = self
            .streams
            .get("fulls")
            .unwrap()
            .len()
            .saturating_sub(fulls_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(fulls)
                .scroll((fulls_offset as u16, 0))
                .block(Block::bordered().title("Full Books")),
            fulls_area,
        );

        let partials = self
            .streams
            .get("partials")
            .unwrap()
            .iter()
            .filter_map(|stream| {
                if let StreamObject::Event(event) = stream {
                    let time = Self::timestamp_to_string(event.receive_time);

                    let partial = if let EventType::PartialOrderBook(ob) = &event.event {
                        ob
                    } else {
                        panic!("Expected PartialOrderBook event")
                    };

                    let bids_len = partial.bids.len();
                    let asks_len = partial.asks.len();

                    Some(format!(
                        "{} | {:<10} | Bids: {:>4} | Asks: {:>4}",
                        time, event.symbol, bids_len, asks_len
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let partials_offset = self
            .streams
            .get("partials")
            .unwrap()
            .len()
            .saturating_sub(partials_area.height as usize - 2);
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

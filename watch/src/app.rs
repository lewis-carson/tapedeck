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
    sync::mpsc::{Receiver, channel},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct App {
    running: bool,

    partials: Vec<datatypes::Event>,
    fulls: Vec<datatypes::Event>,
    orders: Vec<datatypes::Event>,
    fills: Vec<datatypes::Event>,
    worlds: Vec<datatypes::world_builder::World>,

    event_stream: Receiver<datatypes::Event>,
    world_stream: Receiver<datatypes::world_builder::World>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            partials: Vec::new(),
            fulls: Vec::new(),
            orders: Vec::new(),
            fills: Vec::new(),
            worlds: Vec::new(),
            event_stream: channel().1,
            world_stream: channel().1,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        terminal.clear().unwrap();

        // Flatten the new_stream logic for "fulls"
        {
            let (tx, rx) = channel();
            self.event_stream = rx;

            thread::spawn(move || {
                let command = "tail -fq data/*";
                for line in Self::launch_command(command) {
                    let line: datatypes::Event = serde_json::from_str(&line).unwrap();

                    tx.send(line).ok();
                }
            });
        }

        // Flatten the new_stream logic for "world"
        {
            let (tx, rx) = channel();
            self.world_stream = rx;

            thread::spawn(move || {
                let command = "tail -fq data/*";

                let world_builder = datatypes::world_builder::WorldBuilder::new(Box::new(
                    Self::launch_command(command).map(|line| {
                        let event: datatypes::Event = serde_json::from_str(&line).unwrap();
                        Ok(event)
                    }),
                ));

                for world in world_builder {
                    tx.send(world.unwrap()).ok();
                }
            });
        }

        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
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
        for event in self.event_stream.try_iter() {
            match event.event {
                EventType::PartialOrderBook(_) => {
                    self.partials.push(event);
                }
                EventType::FullOrderBook(_) => {
                    self.fulls.push(event);
                }
                EventType::OrderTradeEvent(_) => {
                    self.orders.push(event);
                }
                EventType::AccountInformation(_) => {}
                EventType::OpenOrders(_) => {}
            }
        }

        // cut streams down to 100 elements
        for (_, stream) in &mut [
            ("partials", &mut self.partials),
            ("fulls", &mut self.fulls),
            ("open_orders", &mut self.orders),
            ("fills", &mut self.fills),
        ] {
            if stream.len() > 100 {
                stream.drain(0..stream.len() - 100);
            }
        }

        let master_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
            .split(frame.area());

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(3)])
            .split(master_layout[1]);

        let main_right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .split(main_layout[1]);

        let main_right_top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(main_right[0]);

        let main_right_bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
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
            .worlds
            .last()
            .map(|world| {
                world
                    .account_information
                    .clone()
                    .map(|account| {
                        account
                            .balances
                            .into_iter()
                            .map(|balance| format!("{}: {}", balance.asset, balance.free))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or("No balances in world".to_string())
            })
            .unwrap_or("No world yet".to_string());

        frame.render_widget(
            Paragraph::new(world).block(Block::bordered().title("World")),
            world_area,
        );

        let mut opened = vec![];
        let mut filled = vec![];

        let orders = self.orders.iter().for_each(|event| {
            if let EventType::OrderTradeEvent(order_event) = &event.event {
                let exec = order_event.execution_type.clone();

                if exec == "NEW" {
                    opened.push(format!("{} | {}", Self::timestamp_to_string(event.receive_time), event.symbol));
                } else {
                    filled.push(format!("{} | {}", Self::timestamp_to_string(event.receive_time), event.symbol));
                }
            }
        });

        let opened_offset = opened
            .len()
            .saturating_sub(orders_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(opened.join("\n"))
                .block(Block::bordered().title("Orders"))
                .scroll((opened_offset as u16, 0)),
            orders_area,
        );

        let fills_offset = filled
            .len()
            .saturating_sub(fills_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(filled.join("\n"))
                .block(Block::bordered().title("Fills"))
                .scroll((fills_offset as u16, 0)),
            fills_area,
        );

        // map all of fulls to FullOrderBook vector
        let fulls = self
            .fulls
            .iter()
            .filter_map(|event| {
                let time = Self::timestamp_to_string(event.receive_time);
                Some(format!("{} | {}", time, event.symbol))
            })
            .collect::<Vec<_>>();

        let fulls_offset = self
            .fulls
            .len()
            .saturating_sub(fulls_area.height as usize - 2);

        frame.render_widget(
            Paragraph::new(fulls.join("\n"))
                .scroll((fulls_offset as u16, 0))
                .block(Block::bordered().title("Full Books")),
            fulls_area,
        );

        let partials = self
            .partials
            .iter()
            .filter_map(|event| {
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
            })
            .collect::<Vec<_>>()
            .join("\n");

        let partials_offset = self
            .partials
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

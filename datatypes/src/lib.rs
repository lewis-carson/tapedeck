use binance::model::{DepthOrderBookEvent, OrderBook};
use std::io::Write;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum EventType {
    FullOrderBook(OrderBook),
    PartialOrderBook(DepthOrderBookEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Event {
    pub symbol: String,
    pub receive_time: u64,
    pub event: EventType,
}

impl Event {
    pub fn new(symbol: String, receive_time: u64, event: EventType) -> Self {
        Self {
            symbol,
            receive_time,
            event,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Snapshot {
    events: Vec<Event>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        // write each element of the vector to the file
        for event in &self.events {
            let serialized = serde_json::to_string(event).unwrap();
            writeln!(file, "{}", serialized)?;
        }
        Ok(())
    }
}


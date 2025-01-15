pub mod reader;
pub mod partial_transformer;
pub mod world_builder;

use binance::model::{AccountInformation, DepthOrderBookEvent, OrderBook, OrderTradeEvent};
use serde::Deserialize;
use core::panic;
use std::io::{BufRead, Write};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum EventType {
    FullOrderBook(OrderBook),
    PartialOrderBook(DepthOrderBookEvent),
    AccountInformation(AccountInformation),
    OpenOrders(Vec<binance::model::Order>),
    OrderTradeEvent(OrderTradeEvent),
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

#[derive(Deserialize)]
struct DataItem {
    date: String,
    value: f64,
    l: f64,
    u: f64,
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

    pub fn read_from_file(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let events: Vec<Event> = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                serde_json::from_str(&line).unwrap()
            })
            .collect();
        Ok(Self { events })
    }

}
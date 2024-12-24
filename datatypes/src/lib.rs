use binance::model::{DepthOrderBookEvent, OrderBook};

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
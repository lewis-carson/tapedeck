use std::{collections::HashMap, fmt::{self, Display}, io, ops::Deref, sync::Arc};
use binance::model::{DepthOrderBookEvent, OrderBook};
use crate::{partial_transformer::PartialTransformer, Event, EventType};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct World {
    pub order_books: HashMap<String, OrderBook>
}

impl World {
    pub fn new() -> Self {
        Self {
            order_books: HashMap::new()
        }
    }

    pub fn update_order_book(&mut self, symbol: String, ob: OrderBook) {
        // insert or update order book
        self.order_books.insert(symbol, ob);
    }
}

pub struct WorldBuilder {
    stream: Box<dyn Iterator<Item = io::Result<Event>>>,
    world: World
}

impl WorldBuilder {
    pub fn new(event_iter: Box<dyn Iterator<Item = io::Result<Event>>>) -> Self {
        let partial_transformer = event_iter;

        Self {
            stream: Box::new(partial_transformer),
            world: World::new()
        }
    }
}

impl Iterator for WorldBuilder {
    type Item = io::Result<World>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = match self.stream.next() {
            Some(Ok(event)) => Some(event),
            Some(Err(e)) => return Some(Err(e)),
            None => return None
        };
        
        let symbol = event.as_ref().unwrap().symbol.clone();

        match event.unwrap().event {
            EventType::PartialOrderBook(_) => {},
            EventType::FullOrderBook(ob) => {
                println!("{}", symbol);
                self.world.update_order_book(symbol, ob);
            },
        };

        Some(Ok(self.world.clone()))
    }
}


use std::{collections::HashMap, fmt::{self, Display}, io, ops::Deref, sync::Arc};
use binance::model::{AccountInformation, DepthOrderBookEvent, OrderBook};
use crate::{partial_transformer::PartialTransformer, Event, EventType};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct World {
    pub order_books: HashMap<String, OrderBook>,
    pub account_information: Option<AccountInformation>,
    pub open_orders: Vec<binance::model::Order>
}

impl World {
    pub fn new() -> Self {
        Self {
            order_books: HashMap::new(),
            account_information: None,
            open_orders: Vec::new()
        }
    }

    pub fn update_order_book(&mut self, symbol: String, ob: OrderBook) {
        // insert or update order book
        self.order_books.insert(symbol, ob);
    }

    pub fn update_account_information(&mut self, account: AccountInformation) {
        self.account_information = Some(account);
    }

    pub fn update_open_orders(&mut self, orders: Vec<binance::model::Order>) {
        // insert or update open orders
        self.open_orders = orders;
    }
}

pub struct WorldBuilder {
    stream: Box<dyn Iterator<Item = io::Result<Event>>>,
    world: World
}

impl WorldBuilder {
    pub fn new(event_iter: Box<dyn Iterator<Item = io::Result<Event>>>) -> Self {
        let partial_transformer = PartialTransformer::new(event_iter);

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
            EventType::PartialOrderBook(_) => panic!("Partial order book found in final event stream"),
            EventType::FullOrderBook(ob) => {
                self.world.update_order_book(symbol, ob);
            },
            EventType::AccountInformation(account) => {
                self.world.update_account_information(account);
            },
            EventType::OpenOrders(orders) => {
                self.world.update_open_orders(orders);
            }
            (_) => ()
        };

        Some(Ok(self.world.clone()))
    }
}


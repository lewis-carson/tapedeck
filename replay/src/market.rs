use crate::{partial_transformer, reader};

use std::{collections::HashMap, hash::Hash, io};

use binance::{futures::model::Order, model::OrderBook};
use datatypes::{Event, EventType};
use reader::{EventGrouper, EventIterator};
use partial_transformer::PartialTransformer;

#[derive(Clone, Debug)]
pub struct Holdings (pub HashMap<String, f64>);

impl Holdings {
    pub fn new_cash(cash: f64) -> Self {
        let mut h = HashMap::new();
        h.insert("USD".to_string(), cash);
        Self(h)
    }

    pub fn total_value(&self, lowest_asks: &HashMap<String, f64>) -> f64 {
        let mut total = 0.0;

        for (symbol, amount) in self.0.iter() {
            let price = lowest_asks.get(symbol).unwrap_or(&0.0);
            total += price * amount;
        }

        total += *self.0.get("USD").unwrap_or(&0.0);

        total
    }
}


pub type OrderBundle = Vec<(String, f64)>;


pub struct Market {
    reader: Box<dyn io::BufRead>,
    pub holdings: Holdings,
}

pub type OrderBookCollection = HashMap<String, OrderBook>;

impl Market {
    pub fn new(reader: Box<dyn io::BufRead>, cash: f64) -> Self {
        Self {
            reader,
            holdings: Holdings::new_cash(cash),
        }
    }

    // takes closure as argument
    pub fn run(mut self, f: impl Fn(&Holdings, &OrderBookCollection) -> OrderBundle) -> impl Iterator<Item = (Holdings, HashMap<String, OrderBook>)> {
        let event_iter = Box::new(EventIterator::new(self.reader));
        let transformed_partials = Box::new(PartialTransformer::new(event_iter));
        let grouped_events = EventGrouper::new(transformed_partials);

        let folded_events = grouped_events.scan(HashMap::new(), |acc, event_group| {
            for event in event_group {
                let ob = match event.event {
                    EventType::FullOrderBook(order_book) => order_book,
                    EventType::PartialOrderBook(_) => panic!("Should not have partial order book events"),
                };

                acc.insert(event.symbol, ob);
            }

            Some(acc.clone())
        });

        folded_events.map(move |chunk| {
            let actions = f(&self.holdings, &chunk);

            for action in actions {
                let symbol = action.0;
                let mut matching_ob = None;

                // find the matching order book
                for (s, ob) in chunk.iter() {
                    if s == &symbol {
                        matching_ob = Some(ob);
                        break;
                    }
                }

                // find lowest ask in order book
                let mut lowest_ask = None;
                if let Some(ob) = matching_ob {
                    for ask in ob.asks.iter() {
                        if lowest_ask.is_none() || ask.price < lowest_ask.unwrap() {
                            lowest_ask = Some(ask.price);
                        }
                    }
                }

                // lowest ask is the price we will pay
                let price = lowest_ask.expect("Cannot find symbol in order book");

                // calculate the cost
                let cost = price * action.1;

                // check if we have enough USD to cover the cost
                let available_usd = *self.holdings.0.get("USD").unwrap();
                let amount_to_buy = if cost > available_usd {
                    available_usd / price
                } else {
                    action.1
                };

                // update holdings
                let new_target_amount = self.holdings.0.get(&symbol).unwrap_or(&0.0) + amount_to_buy;
                self.holdings.0.insert(symbol, new_target_amount);
                self.holdings.0.insert("USD".to_string(), available_usd - (price * amount_to_buy));
            }

            (self.holdings.clone(), chunk.clone())
        })
    }

    pub fn run_without_actions(self) -> impl Iterator<Item = (u64, HashMap<String, OrderBook>)> {
        let event_iter = Box::new(EventIterator::new(self.reader));
        let transformed_partials = Box::new(PartialTransformer::new(event_iter));
        let grouped_events = EventGrouper::new(transformed_partials);

        grouped_events.scan(HashMap::new(), |acc, event_group| {

            let timestamp = event_group.first().unwrap().receive_time;
                
            for event in event_group {
                let ob = match event.event {
                    EventType::FullOrderBook(order_book) => order_book,
                    EventType::PartialOrderBook(_) => panic!("Should not have partial order book events"),
                };

                acc.insert(event.symbol, ob);
            }

            Some((timestamp, acc.clone()))
        })
    }
}

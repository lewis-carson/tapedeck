use std::{collections::HashMap, io};
use binance::model::{DepthOrderBookEvent, OrderBook};
use crate::{Event, EventType};


fn update_full_order_book(ob: &mut OrderBook, dob: &DepthOrderBookEvent) {
    // bids are in descending order
    for bid in dob.bids.iter() {
        // find index of bid in ob.bids
        let index = ob.bids.iter().position(|x| x.price == bid.price);

        match index {
            Some(i) => {
                if bid.qty == 0.0 {
                    ob.bids.remove(i);
                } else {
                    ob.bids[i].qty = bid.qty;
                }
            }
            None => {
                if bid.qty != 0.0 {
                    ob.bids.push(bid.clone());
                    // sort bids in descending order
                    ob.bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                }
            }
        }
    }

    // asks are in ascending order
    for ask in dob.asks.iter() {
        // find index of ask in ob.asks
        let index = ob.asks.iter().position(|x| x.price == ask.price);

        match index {
            Some(i) => {
                if ask.qty == 0.0 {
                    ob.asks.remove(i);
                } else {
                    ob.asks[i].qty = ask.qty;
                }
            }
            None => {
                if ask.qty != 0.0 {
                    ob.asks.push(ask.clone());
                    // sort asks in ascending order
                    ob.asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                }
            }
        }
    }
}

pub struct PartialTransformer {
    event_iter: Box<dyn Iterator<Item = io::Result<Event>>>,
    order_books: HashMap<String, OrderBook>,
}

impl PartialTransformer {
    pub fn new(event_iter: Box<dyn Iterator<Item = io::Result<Event>>>) -> Self {
        Self {
            event_iter: event_iter,
            order_books: HashMap::new(),
        }
    }
}

impl Iterator for PartialTransformer {
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let e = self.event_iter.next();

        if let Some(Ok(ref ev)) = e {
            match ev.event {
                EventType::FullOrderBook(ref ob) => {
                    self.order_books.insert(ev.symbol.clone(), ob.clone());
                }
                EventType::PartialOrderBook(ref dob) => {
                    if let Some(ob) = self.order_books.get_mut(&ev.symbol) {
                        update_full_order_book(ob, dob);
                    }
                }
                _ => {
                    return Some(Ok(ev.clone()));
                }
            }

            // if we have a full order book for this symbol, we can create a new event

            if let Some(ob) = self.order_books.get(&ev.symbol) {      
                Some(Ok(Event::new(
                    ev.symbol.clone(),
                    ev.receive_time,
                    EventType::FullOrderBook(ob.clone()),
                )))
            } else {
                self.next()
            }
        } else {
            // EOF
            None
        }
    }
}

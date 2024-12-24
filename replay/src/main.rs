mod reader;
mod partial_transformer;

use std::io;

use datatypes::EventType;
use reader::{EventGrouper, EventIterator};
use partial_transformer::PartialTransformer;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let event_iter = Box::new(EventIterator::new(reader));
    let transformed_partials = Box::new(PartialTransformer::new(event_iter));
    let grouped_events = EventGrouper::new(transformed_partials);

    for chunk in grouped_events {
        let timestamp = chunk[0].receive_time;

        println!("{timestamp} {:?}", chunk.iter().map(|ev| {
            let spread = match &ev.event {
                EventType::FullOrderBook(ob) => ob.asks[0].price - ob.bids[0].price,
                EventType::PartialOrderBook(dob) => panic!("Unexpected partial order book event: {:?}", dob),
            };
            (&ev.symbol, spread)
        }).collect::<Vec<(&String, f64)>>());
    }
    Ok(())
}

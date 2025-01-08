mod partial_transformer;
mod reader;

use partial_transformer::PartialTransformer;
use reader::EventIterator;

use std::io::{self, BufRead};
use datatypes::EventType::FullOrderBook;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let reader = Box::new(stdin.lock());
    let event_iter = Box::new(EventIterator::new(reader));
    
    let transformed_partials = PartialTransformer::new(event_iter);

    for ob in transformed_partials {
        println!("{}", serde_json::to_string(&ob.unwrap()).unwrap());
    }

    Ok(())
}

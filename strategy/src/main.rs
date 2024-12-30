use replay::{Market, OrderBundle, utils};
use std::{io, sync::Mutex};

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader), 1000.0);

    let flip = Mutex::new(true);

    let mut indicator = utils::Indicator::new();
    
    for (new_holdings, last_events) in market.run(|holdings, events| {
        // Create and return a Vec<OrderBundle>
        
        let mut flip = flip.lock().unwrap();

        let first_symbol = events.keys().next().unwrap();

        if *flip {
            *flip = false;
            vec![(first_symbol.to_string(), 0.01)]
        } else {
            *flip = true;
            vec![(first_symbol.to_string(), -0.01)]
        }
    }) {
        indicator.update(&new_holdings, &last_events);
    };

    indicator.finish();
}

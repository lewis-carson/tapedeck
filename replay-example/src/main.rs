use replay::{Market, OrderBundle, utils};
use std::{io, sync::Mutex};

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader), 1000.0);

    let flip = Mutex::new(true);

    /*let mut indicator = utils::Indicator::new();
    
    for (new_holdings, last_events) in market.run(|holdings, events| {
        // Create and return a Vec<OrderBundle>

        let first_symbol = events.keys().next().unwrap();
        vec![(first_symbol.to_string(), 0.01)]
    }) {
        indicator.update(&new_holdings, &last_events);
        // sleep 1 second
        //std::thread::sleep(std::time::Duration::from_millis(100));
    };

    indicator.finish();*/

    for book in market.run_without_actions() {
        // get min ask
        let min_ask = book.1.iter().map(|(symbol, ob)| {
            let ask = ob.asks.first().unwrap().price;
            (symbol.clone(), ask)
        }).map(|(_, price)| price).next().unwrap();

        println!("{},{:?}", book.0, min_ask);
    }
}

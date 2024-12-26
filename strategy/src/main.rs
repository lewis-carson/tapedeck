use replay::{Market, OrderBundle};
use std::{io, sync::Mutex};

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader), 1000.0);
    
    let final_holdings = market.run(|holdings, events| {
        // Create and return a Vec<OrderBundle>
        vec![("BTCUSDT".to_string(), 0.001)]
    });

    println!("{:?}", final_holdings);
}

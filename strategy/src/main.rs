use replay::Market;
use std::io;

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader));
    
    market.run(|c| {
        println!("{:?}", c.into_iter().map(|s| &s.symbol).collect::<Vec<&String>>());
    });
}

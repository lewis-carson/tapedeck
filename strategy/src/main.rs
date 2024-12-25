use replay::Market;
use std::{io, sync::Mutex};

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader));

    let mut first_timestamp = Mutex::new(0);
    let mut last_timestamp = Mutex::new(0);
    
    market.run(|c| {
        let mut first_timestamp = first_timestamp.lock().unwrap();
        let mut last_timestamp = last_timestamp.lock().unwrap();

        if *first_timestamp == 0 {
            *first_timestamp = c[0].receive_time;
        }

        *last_timestamp = c[0].receive_time;
    });

    let first_timestamp = first_timestamp.lock().unwrap();
    let last_timestamp = last_timestamp.lock().unwrap();
    
    println!("diff {}", *last_timestamp - *first_timestamp);
}

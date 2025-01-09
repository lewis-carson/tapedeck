use replay::{Market, OrderBundle, utils};
use std::{io, sync::Mutex};
use datatypes::Snapshot;

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader), 1000.0);

    let mut snapshot = Snapshot::new();
    let mut n = 0;

    for ev in market.run_raw_events() {
        n += 1;
    }

    println!("Processed {} events", n);

    snapshot.write_to_file("snapshot.json").unwrap();
}

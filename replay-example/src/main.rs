use replay::{Market, OrderBundle, utils};
use std::{io, sync::Mutex};
use datatypes::Snapshot;
use datatypes::EventType::FullOrderBook;

fn main() {
    let stdin = io::stdin();
    let reader = stdin.lock();

    let market: Market = Market::new(Box::new(reader), 1000.0);

    let mut snapshot = Snapshot::new();
    let mut n = 0;

    for ev in market.run_raw_events() {
        if n == 100 {
            break;
        }

        if ev.symbol == "BTCUSDT"{
            snapshot.add_event(ev);
        }

        n += 1;
    }

    snapshot.display().unwrap();

}

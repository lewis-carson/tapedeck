use binance::futures::model::BookTickers::AllBookTickers;
use binance::{
    api::Binance,
    market::Market,
    model::{DepthOrderBookEvent, OrderBook},
    websockets::*,
};
use serde;
use std::{io::Write, sync::atomic::AtomicBool};

const CORRECTION_INTERVAL_MINUTES: i64 = 5;
const N_SYMBOLS: usize = 750;

#[derive(serde::Serialize)]
struct FullOrderBook {
    symbol: String,
    receive_time: u64,
    order_book: OrderBook,
    is_partial: bool,
}

#[derive(serde::Serialize)]
struct PartialOrderBook {
    symbol: String,
    receive_time: u64,
    order_book: DepthOrderBookEvent,
    is_partial: bool,
}

fn main() {
    // open lines in symbols file as vec
    let market: Market = Binance::new(None, None);

    let symbols = match market.get_all_book_tickers() {
        Ok(answer) => {
            let AllBookTickers(list) = answer;
            list.into_iter()
                .map(|ticker| ticker.symbol.to_lowercase())
                .collect::<Vec<String>>()
        }
        Err(e) => panic!("Error: {:?}", e),
    };

    let symbols = symbols.iter().take(N_SYMBOLS).collect::<Vec<&String>>();

    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let depth = symbols
        .iter()
        .map(|symbol| format!("{}@depth@100ms", symbol))
        .collect::<Vec<String>>();

    let mut full_book_correction_schedule = vec![chrono::Utc::now(); N_SYMBOLS];
    // spread full book correction schedule over the next 10 minutes -- equally space over N_SYMBOLS symbols
    for i in 0..N_SYMBOLS {
        full_book_correction_schedule[i] = full_book_correction_schedule[i]
            .checked_add_signed(chrono::Duration::minutes(
                CORRECTION_INTERVAL_MINUTES * i as i64 / N_SYMBOLS as i64,
            ))
            .unwrap();
    }

    let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
        match event {
            // 24hr rolling window ticker statistics for all symbols that changed in an array.
            WebsocketEvent::DepthOrderBook(depth_order_book) => {
                let recv_time = chrono::Utc::now().timestamp_millis() as u64;

                let depth_order_book = PartialOrderBook {
                    symbol: depth_order_book.symbol.to_string(),
                    receive_time: recv_time,
                    order_book: depth_order_book,
                    is_partial: true,
                };

                // append under data/{symbol} directory
                let symbol = &depth_order_book.symbol;

                let file_name = format!("data/{}.json", symbol);

                // create file if not exists
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&file_name)
                    .unwrap();

                // serialise to json
                let depth_order_book = serde_json::to_string(&depth_order_book).unwrap();

                // write to file
                file.write_all(depth_order_book.as_bytes()).unwrap();
                file.write_all(b"\n").unwrap();

                // check if full order book correction is due
                let index = symbols
                    .iter()
                    .position(|s| **s == *symbol.to_lowercase())
                    .unwrap();
                let correction_due = chrono::Utc::now() > full_book_correction_schedule[index];

                if correction_due {
                    println!("Order correction for {}", symbol);
                    // send order correction
                    let answer = match market.get_custom_depth(symbol, 500) {
                        Ok(answer) => answer,
                        Err(e) => panic!("Error: {}", e),
                    };

                    let answer = FullOrderBook {
                        symbol: symbol.to_string(),
                        receive_time: recv_time,
                        order_book: answer,
                        is_partial: false,
                    };

                    // serialise to json
                    let depth_order_book = serde_json::to_string(&answer).unwrap();

                    // write to file
                    file.write_all(depth_order_book.as_bytes()).unwrap();
                    file.write_all(b"\n").unwrap();

                    // get time 10 minutes from now
                    full_book_correction_schedule[index] = chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::minutes(CORRECTION_INTERVAL_MINUTES))
                        .unwrap();
                }
            }
            _ => (),
        };

        Ok(())
    });

    web_socket.connect_multiple_streams(&depth).unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running) {
        match e {
            err => {
                println!("Error: {:?}", err);
            }
        }
    }
}

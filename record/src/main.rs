use binance::futures::model::BookTickers::AllBookTickers;
use binance::{
    api::Binance,
    market::Market,
    websockets::*,
};

use std::{io::Write, sync::atomic::AtomicBool};
use tokio::sync::mpsc;
use tokio::task;

const CORRECTION_INTERVAL: i64 = 100;
const N_SYMBOLS: usize = 750;

use datatypes::{Event, EventType};


#[tokio::main]
async fn main() {
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

    let mut ticks_since_last_correction = vec![0; N_SYMBOLS];
    
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn a background task
    let handle = task::spawn(async move {
        while let Some(symbol) = rx.recv().await {
            let recv_time = chrono::Utc::now().timestamp_millis() as u64;

            let file_name = format!("../data/{}.json", symbol);

            // create file if not exists
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&file_name)
                .unwrap();

            // send order correction
            let answer = match market.get_custom_depth(&symbol, 500) {
                Ok(answer) => answer,
                Err(e) => panic!("Error: {}", e),
            };

            let answer = Event::new(
                symbol.clone(),
                recv_time,
                EventType::FullOrderBook(answer),
            );

            // serialise to json
            let depth_order_book = serde_json::to_string(&answer).unwrap();

            // write to file
            file.write_all(depth_order_book.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
    });

    let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
        match event {
            // 24hr rolling window ticker statistics for all symbols that changed in an array.
            WebsocketEvent::DepthOrderBook(depth_order_book) => {
                let recv_time = chrono::Utc::now().timestamp_millis() as u64;
                let symbol = depth_order_book.symbol.clone();

                let depth_order_book = Event::new(
                    depth_order_book.symbol.clone(),
                    recv_time,
                    EventType::PartialOrderBook(depth_order_book),
                );

                // append under ../data/{symbol} directory
                let file_name = format!("../data/{}.json", symbol);

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

                ticks_since_last_correction[index] += 1;

                if ticks_since_last_correction[index] == CORRECTION_INTERVAL {
                    
                    let _ = futures::executor::block_on(tx.send(symbol.clone()));

                    ticks_since_last_correction[index] = 0;
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
    handle.await.unwrap();
}

mod spinner;

use binance::futures::model::BookTickers::AllBookTickers;
use binance::{
    api::Binance,
    market::Market,
    websockets::*,
};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::Write, sync::atomic::AtomicBool};
use crossfire::mpsc;
use tokio::task;
use std::env;

use indicatif::{ProgressBar, ProgressStyle};
use spinner::*;

const CORRECTION_INTERVAL: i64 = 100;
const N_SYMBOLS: usize = 750;

use datatypes::{Event, EventType};


#[tokio::main]
async fn main() {
    
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(1_000));
    pb.set_style(spinner());
    pb.set_message("Recording");
     

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <output-dir>", args[0]);
        std::process::exit(1);
    }
    let output_dir = args[1].clone();

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
    
    let (tx, rx) = mpsc::bounded_tx_blocking_rx_future::<String>(N_SYMBOLS);

    let n_data_points = Arc::new(Mutex::new(0));

    // Spawn a background task
    let handle = task::spawn(async move {
        let output_dir = args[1].clone();
        
        while let Ok(symbol) = rx.recv().await {
            let recv_time = chrono::Utc::now().timestamp_millis() as u64;

            let file_name = format!("{}/{}.json", output_dir, symbol);

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

            // increment data points counter
            //*n_data_points.lock().unwrap() += 1;
        }
    });

    let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
        //println!("Event n: {:?}", n_data_points);
        {
            let data_points = n_data_points.lock().unwrap();
            pb.set_message(format!("Recording [{} data points]", data_points));
        }

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

                // append under {output_dir}/{symbol} directory
                let file_name = format!("{}/{}.json", output_dir, symbol);

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
                    let _ = tx.send(symbol.clone());

                    ticks_since_last_correction[index] = 0;
                }

                // increment data points counter
                {
                    *n_data_points.lock().unwrap() += 1;
                }
            }
            _ => panic!("Error: {:?}", event),
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

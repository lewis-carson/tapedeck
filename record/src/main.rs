mod spinner;

use binance::futures::model::BookTickers::AllBookTickers;
use binance::{api::Binance, market::Market, websockets::*};

use crossfire::mpsc;
use human_repr::HumanCount;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io::Write, sync::atomic::AtomicBool};
use tokio::task;

use indicatif::ProgressBar;
use spinner::*;

const CORRECTION_INTERVAL: i64 = 50;
const N_SYMBOLS: usize = 100;
const CORRECTION_TIMEOUT: u64 = 500;

use datatypes::{Event, EventType};

#[derive(Clone, Copy)]
struct RunTimeStats {
    n_data_points: usize,
    bytes_written: usize,
    n_full_books: usize,
    start_time: u64,
}

impl RunTimeStats {
    fn new() -> Self {
        RunTimeStats {
            n_data_points: 1,
            bytes_written: 0,
            n_full_books: 1,
            start_time: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    fn elapsed_time(&self) -> u64 {
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        current_time - self.start_time
    }
}

impl Display for RunTimeStats {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // full books / symbol
        let fb_per_symbol_per_minute = (N_SYMBOLS as f64 * (self.elapsed_time() as f64 / 60_000.0)) / self.n_full_books as f64;
        write!(
            f,
            "{}",
            format!(
                "[Symbols:   {}] [Symbolm/fb {:.5}] [Samples: {:>7}] [Written: {:>8}]",
                N_SYMBOLS,
                fb_per_symbol_per_minute.to_string(),
                self.n_data_points.human_count_bare().to_string(),
                self.bytes_written.human_count_bytes().to_string()
            )
        )
    }
}

#[tokio::main]
async fn main() {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(1_000));
    pb.set_style(spinner());

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

    let runtime_stats = Arc::new(Mutex::new(RunTimeStats::new()));
    let bg_runtime_stats = runtime_stats.clone();
    
    // Spawn a background task
    let handle = task::spawn(async move {
        let output_dir = args[1].clone();

        while let Ok(symbol) = rx.recv().await {
            // sleep to avoid binance kicking us off
            tokio::time::sleep(Duration::from_millis(CORRECTION_TIMEOUT)).await;

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
                Ok(answer) => {
                    answer},
                Err(_) => {
                    //println!("Error: {:?}", symbol);
                    continue;
                },
            };

            let answer = Event::new(symbol.clone(), recv_time, EventType::FullOrderBook(answer));

            // serialise to json
            let depth_order_book = serde_json::to_string(&answer).unwrap();

            // write to file
            file.write_all(depth_order_book.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();

            // increment data points counter
            // increment runtime stats
            {
                let mut stats = bg_runtime_stats.lock().unwrap();
                stats.n_data_points += 1;
                stats.n_full_books += 1;
                stats.bytes_written += depth_order_book.as_bytes().len();
            }
        }
    });

    let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
        //println!("Event n: {:?}", n_data_points);
        {
            let runtime_stats = runtime_stats.lock().unwrap();
            pb.set_message(runtime_stats.to_string());
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
                let bytes_written = depth_order_book.as_bytes().len();

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

                // increment runtime stats
                {
                    let mut stats = runtime_stats.lock().unwrap();
                    stats.n_data_points += 1;
                    stats.bytes_written += bytes_written;
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

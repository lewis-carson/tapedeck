use binance::{api::Binance, market::Market, model::{DepthOrderBookEvent, OrderBook}, websockets::*};
use std::{io::Write, sync::atomic::AtomicBool};
use binance::futures::model::BookTickers::AllBookTickers;
use serde;

const ORDER_CORRECTION_INTERVAL: i32 = 500;

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
            list.into_iter().map(|ticker| ticker.symbol.to_lowercase()).collect::<Vec<String>>()
        },
        Err(e) => panic!("Error: {:?}", e),
    };
    
    let symbols = symbols.iter().take(750).collect::<Vec<&String>>();

    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let depth = symbols.iter().map(|symbol| format!("{}@depth@100ms", symbol)).collect::<Vec<String>>();

    let mut ticks_since_full_book = [0; 750];

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

            // update ticks_since_full_book
            let index = symbols.iter().position(|s| **s == symbol.to_lowercase()).unwrap();
            ticks_since_full_book[index] += 1;

            if ticks_since_full_book[index] >= ORDER_CORRECTION_INTERVAL {
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

                ticks_since_full_book[index] = 0;
            }

        },
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
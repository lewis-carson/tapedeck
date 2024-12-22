use binance::{api::Binance, market::Market, websockets::*};
use std::{io::Write, sync::atomic::AtomicBool};
use binance::futures::model::BookTickers::AllBookTickers;

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

    let mut web_socket = WebSockets::new(|event: WebsocketEvent| {

	match event {
        // 24hr rolling window ticker statistics for all symbols that changed in an array.
	    WebsocketEvent::DepthOrderBook(depth_order_book) => {
            // append under data/{symbol} directory
            let symbol = &depth_order_book.symbol;

            let file_name = format!("data/{}.json", symbol);

            // create file if not exists
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&file_name);
            
            // serialise to json
            let depth_order_book = serde_json::to_string(&depth_order_book).unwrap();

            // write to file
            match file {
                Ok(mut file) => {
                    file.write_all(depth_order_book.as_bytes()).unwrap();
                    file.write_all(b"\n").unwrap();
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
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
use binance::config::Config;
use binance::model::{BalanceUpdateEvent, OrderTradeEvent};
use binance::userstream::*;
use binance::{api::*, model::AccountUpdateEvent};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use tungstenite::{Message, connect};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Events {
    BalanceUpdateEvent(BalanceUpdateEvent),
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(OrderTradeEvent),
}

fn write_trade_event(output_dir: &str, event: &OrderTradeEvent) -> Result<(), std::io::Error> {
    let file_name = format!("{}/account.json", output_dir);
    let recv_time = chrono::Utc::now().timestamp_millis() as u64;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&file_name)
        .unwrap();

    let event = datatypes::Event::new("account".to_string(), recv_time, datatypes::EventType::OrderTradeEvent(event.clone()));

    let event = serde_json::to_string(&event).unwrap();
    let to_write = format!("{}\n", event);

    file.write_all(to_write.as_bytes())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let output_dir = args[1].clone();

    let api_key_user = env::var("BINANCE_API_KEY").ok();
    let api_key_user_secret = env::var("BINANCE_API_SECRET_KEY").ok();
    let rest_api_endpoint = env::var("REST_ENDPOINT").unwrap();

    let config = Config::default().set_rest_api_endpoint(rest_api_endpoint);

    let user_stream: UserStream =
        Binance::new_with_config(api_key_user, api_key_user_secret, &config);

    loop {
        if let Ok(answer) = user_stream.start() {
            let listen_key = answer.listen_key;

            let endpoint = "wss://stream.testnet.binance.vision:9443/ws/".to_string() + &listen_key;

            let (mut socket, _) = connect(&endpoint).expect("Can't connect");

            // Handle WebSocket messages
            while let Ok(message) = socket.read() {
                let text = match message {
                    Message::Text(text) => text,
                    _ => continue,
                };
                let value: serde_json::Value = serde_json::from_str(&text).unwrap();

                let event = serde_json::from_value::<Events>(value);

                if let Ok(event) = event {
                    match event {
                        Events::BalanceUpdateEvent(e) => {
                            panic!("AccountUpdateEvent: {:?}", e);
                        }
                        Events::AccountUpdateEvent(e) => {
                            panic!("AccountUpdateEvent: {:?}", e);
                        }
                        Events::OrderTradeEvent(e) => {
                            write_trade_event(&output_dir, &e).unwrap();
                        }
                    }
                }
            }
        } else {
            println!("Not able to start an User Stream (Check your API_KEY)");
        }

        // sleep for 5 seconds to avoid getting kicked off
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

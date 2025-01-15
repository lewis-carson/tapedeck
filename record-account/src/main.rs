use binance::account::Account;
use binance::api::*;
use binance::config::Config;
use binance::userstream::*;
use binance::websockets::*;
use datatypes::Event;
use datatypes::EventType;
use std::env;
use std::io::Write;
use std::sync::atomic::AtomicBool;

fn main() {
    let args: Vec<String> = env::args().collect();
    let output_dir = args[1].clone();

    let api_key_user = env::var("BINANCE_API_KEY").ok();
    let api_key_user_secret = env::var("BINANCE_API_SECRET_KEY").ok();
    let ws_endpoint = env::var("WS_ENDPOINT").unwrap();
    let rest_api_endpoint = env::var("REST_ENDPOINT").unwrap();

    let config = Config::default()
        .set_ws_endpoint(ws_endpoint)
        .set_rest_api_endpoint(rest_api_endpoint);

    let account: Account = Binance::new_with_config(api_key_user, api_key_user_secret, &config);

    loop {
        match account.get_all_open_orders() {
            Ok(answer) => {
                let event = Event::new(
                    "account".to_string(),
                    chrono::Utc::now().timestamp_millis() as u64,
                    EventType::OpenOrders(answer),
                );

                let file_name = format!("{}/account.json", output_dir);

                // create file if not exists
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&file_name)
                    .unwrap();

                // serialise to json
                let account_info = serde_json::to_string(&event).unwrap();

                // write to file
                let to_write = format!("{}\n", account_info);
                file.write_all(to_write.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        // sleep to avoid binance kicking us off
        std::thread::sleep(std::time::Duration::from_millis(1000));
        match account.get_account() {
            Ok(answer) => {
                let event = Event::new(
                    "account".to_string(),
                    chrono::Utc::now().timestamp_millis() as u64,
                    EventType::AccountInformation(answer),
                );

                let file_name = format!("{}/account.json", output_dir);

                // create file if not exists
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(&file_name)
                    .unwrap();

                // serialise to json
                let account_info = serde_json::to_string(&event).unwrap();

                // write to file
                let to_write = format!("{}\n", account_info);
                file.write_all(to_write.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }

        // sleep to avoid binance kicking us off
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

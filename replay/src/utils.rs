use std::{collections::HashMap, time::{Duration, Instant}};

use crate::market::Holdings;
use binance::model::OrderBook;
use indicatif::{ProgressBar, ProgressStyle};
use human_repr::{HumanCount, HumanDuration};

pub struct Indicator {
    pb: ProgressBar,
    first_update: Option<u64>,
}

impl Indicator {
    pub fn new() -> Self {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(1_000));

        let spinner = ProgressStyle::with_template("{spinner} Backtesting... {msg}")
            .unwrap()
            .tick_strings(&[
                "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦",
                "⣮", "⣶", "⣷", "⣿", "⡿", "⠿", "⢟", "⠟", "⡛", "⠛", "⠫", "⢋", "⠋", "⠍", "⡉", "⠉",
                "⠑", "⠡", "⢁",
            ]);

        pb.set_style(spinner);

        Self { pb, first_update: None }
    }

    pub fn update(&mut self, step: &Holdings, last_events: &HashMap<String, OrderBook>) {        
        let lowest_asks = last_events
            .iter()
            .map(|(symbol, ob)| {
                let ask = ob.asks.first().unwrap().price;
                (symbol.clone(), ask)
            })
            .collect::<HashMap<_, _>>();

        let total_value = step.total_value(&lowest_asks);

        // find holdings that are not zero
        let holdings = step.0.iter().filter(|(_, v)| **v != 0.0).count() - 1;
        let symbols = last_events.len();

        let cash_percent = step.0.get("USD").unwrap_or(&0.0) / total_value * 100.0;

        let msg = format!(
            "[{}/{} symbols] [{:>5} total value] [{:.3}% cash]",
            holdings,
            symbols,
            total_value as u64,
            cash_percent
        );
        // do something with step
        self.pb.set_message(
            msg.to_string()
        );
    }

    pub fn finish(&self) {
        self.pb.finish();
    }
}

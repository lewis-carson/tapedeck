pub mod reader;

use binance::model::{DepthOrderBookEvent, OrderBook};
use serde::Deserialize;
use core::panic;
use std::io::{BufRead, Write};

use charming::{
    component::{Axis, Grid, Legend, Title, VisualMap},
    datatype::{CompositeValue, DataFrame},
    df,
    element::*,
    series::{Heatmap, Line, Pie, PieRoseType},
    Chart, HtmlRenderer,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum EventType {
    FullOrderBook(OrderBook),
    PartialOrderBook(DepthOrderBookEvent),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Event {
    pub symbol: String,
    pub receive_time: u64,
    pub event: EventType,
}

impl Event {
    pub fn new(symbol: String, receive_time: u64, event: EventType) -> Self {
        Self {
            symbol,
            receive_time,
            event,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Snapshot {
    events: Vec<Event>,
}

#[derive(Deserialize)]
struct DataItem {
    date: String,
    value: f64,
    l: f64,
    u: f64,
}

impl Snapshot {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        // write each element of the vector to the file
        for event in &self.events {
            let serialized = serde_json::to_string(event).unwrap();
            writeln!(file, "{}", serialized)?;
        }
        Ok(())
    }

    pub fn read_from_file(path: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let events: Vec<Event> = reader
            .lines()
            .map(|line| {
                let line = line.unwrap();
                serde_json::from_str(&line).unwrap()
            })
            .collect();
        Ok(Self { events })
    }

    pub fn display(&self) -> std::io::Result<()> {
        let data = self.events.clone().into_iter().map(|ev| {
            let receive_date_string = ev.receive_time.to_string();
            let ev = if let EventType::FullOrderBook(order_book) = &ev.event {
                order_book
            } else {
                panic!("Expected FullOrderBook event");
            };

            let best_bid = ev.bids[0].price;
            let best_ask = ev.asks[0].price;

            println!("{}: {}", receive_date_string, (best_bid + best_ask) / 2.0);

            DataItem {
                date: receive_date_string,
                value: (best_bid + best_ask) / 2.0,
                l: best_bid,
                u: best_ask,
            }
        }).collect::<Vec<DataItem>>();

        let base = -data
            .iter()
            .fold(f64::INFINITY, |min, val| f64::floor(f64::min(min, val.l)));

        let chart = Chart::new()
        .title(
            Title::new()
                .text("Confidence Band")
                .subtext("Example in MetricsGraphics.js")
                .left("center"),
        )
        .grid(Grid::new().left("3%").right("4%").bottom("10%").contain_label(true))
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(data.iter().map(|x| x.date.clone()).collect())
                .axis_label(
                    AxisLabel::new().formatter("{value}")
                )
                .boundary_gap(false)
        )
        .y_axis(
            Axis::new()
                .axis_label(AxisLabel::new().formatter("{value}"))
                .axis_pointer(
                    AxisPointer::new().label(
                        Label::new().formatter("{value}")
                    )
                ).split_number(3)
        )
        .series(
            Line::new()
                .name("L")
                .data(data.iter().map(|x| x.l + base).collect())
                .line_style(LineStyle::new().opacity(0))
                .stack("confidence-band")
                .symbol(Symbol::None)
        )
        .series(
            Line::new()
                .name("U")
                .data(data.iter().map(|x| x.u - x.l).collect())
                .line_style(LineStyle::new().opacity(0))
                .area_style(AreaStyle::new().color("#ccc"))
                .stack("confidence-band")
                .symbol(Symbol::None)
        )
        .series(
            Line::new()
                .data(data.iter().map(|x| x.value + base).collect())
                .item_style(ItemStyle::new().color("#333"))
                .show_symbol(false));

        let mut renderer = HtmlRenderer::new("test", 1000, 800);
        renderer.save(&chart, "/tmp/test.html").unwrap();

        // open the file in the default browser
        opener::open("/tmp/test.html").unwrap();

        Ok(())
    }
}

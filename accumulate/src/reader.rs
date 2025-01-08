use std::io::{self, BufRead};
use serde_json;
use datatypes::Event;

pub struct EventIterator<R: BufRead> {
    reader: R,
}

impl<R: BufRead> EventIterator<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead> Iterator for EventIterator<R> {
    type Item = io::Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.by_ref().lines().next().map(|line_result| {
            line_result.map(|line| {
                serde_json::from_str(&line).unwrap()
            })
        })
    }
}

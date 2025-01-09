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

pub struct EventGrouper {
    event_iter: Box<dyn Iterator<Item = io::Result<Event>>>,
    peeked_event: Option<Event>,
}

impl EventGrouper {
    pub fn new(event_iter: Box<dyn Iterator<Item = io::Result<Event>>>) -> Self {
        Self {
            event_iter: event_iter,
            peeked_event: None,
        }
    }
}

impl Iterator for EventGrouper {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let first_event = match self.peeked_event.take() {
            Some(ev) => ev,
            None => self.event_iter.next()?.ok()?,
        };
        let current_time = first_event.receive_time;
        let mut chunk = vec![first_event];

        while let Some(Ok(ev)) = self.event_iter.next() {
            if ev.receive_time == current_time {
                chunk.push(ev);
            } else {
                self.peeked_event = Some(ev);
                break;
            }
        }
        Some(chunk)
    }
}

impl Drop for EventGrouper {
    fn drop(&mut self) {
        // Handle closing the pipe gracefully if necessary
    }
}

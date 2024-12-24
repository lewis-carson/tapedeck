use crate::{partial_transformer, reader};

use std::io;

use datatypes::{Event, EventType};
use reader::{EventGrouper, EventIterator};
use partial_transformer::PartialTransformer;

pub struct Market {
    reader: Box<dyn io::BufRead>,
}

impl Market {
    pub fn new(reader: Box<dyn io::BufRead>) -> Self {
        Self {
            reader,
        }
    }

    // takes closure as argument
    pub fn run(self, f: impl Fn(&Vec<Event>) -> ()) {
        let event_iter = Box::new(EventIterator::new(self.reader));
        let transformed_partials = Box::new(PartialTransformer::new(event_iter));
        let grouped_events = EventGrouper::new(transformed_partials);

        for chunk in grouped_events {
            f(&chunk);
        }
    }
}

use datatypes::partial_transformer::PartialTransformer;
use datatypes::reader::EventIterator;
use datatypes::world_builder::WorldBuilder;
use std::io::Write;

use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let reader = Box::new(stdin.lock());
    let event_iter = Box::new(EventIterator::new(reader));
    
    let world_builder = WorldBuilder::new(event_iter);

    for ob in world_builder {
        // instead of println, do this to prevent broken pipe errors
        // the error still happens, we just ignore it
        let mut stdout = io::stdout();
        //let _ = write!(stdout, "{}\n", serde_json::to_string(&ob.unwrap()).unwrap());
    }

    Ok(())
}

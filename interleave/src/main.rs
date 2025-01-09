use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::u64;
use datatypes::Event;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the data directory
    path: String,
}

struct LineGenerator {
    reader: BufReader<File>,
}

impl LineGenerator {
    fn new(path: PathBuf) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }

    fn next_line(&mut self) -> Option<Event> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                if line.trim().is_empty() {
                    return self.next_line();
                }
                serde_json::from_str(&line).ok()
            }
            Err(_) => None,
        }
    }
}

fn get_files(path: &str) -> io::Result<Vec<PathBuf>> {
    Ok(std::fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let files = get_files(&args.path)?;

    let mut generators: Vec<LineGenerator> = files
        .into_iter()
        .filter_map(|path| LineGenerator::new(path).ok())
        .collect();

    let mut line_data: Vec<Option<Event>> = generators
        .iter_mut()
        .map(|gen| gen.next_line())
        .collect();

    while line_data.iter().any(|data| data.is_some()) {
        let times: Vec<u64> = line_data
            .iter()
            .map(|data| data.as_ref().map_or(u64::MAX, |d| d.receive_time))
            .collect();

        let smallest_time_index = times
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(index, _)| index)
            .unwrap();

        if let Some(data) = &line_data[smallest_time_index] {
            println!("{}", serde_json::to_string(data).unwrap());
        }

        line_data[smallest_time_index] = generators[smallest_time_index].next_line();
    }

    Ok(())
}

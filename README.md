# tapedeck

A toolkit to record and replay millisecond-level Binance order book data.

tapedeck is built to be modular - there are three ways to interact with the tools provided.

- `record`: records timestamped order book data to data/ directory. Serialised using json to be easily read by other tools
- `interleave`: takes n order book data files and interleaves them based on timestamp. Useful combining data from multiple ticker symbols
- `replay`: deserialises the data stream, builds a full order book from the partial updates and replays the order book back (in the same order as the original data)

For example, you could:
- `record` to record order book data into a directory
- `record > interleave` to playback multiple order books at the same time to stdout
- `record > interleave > strategy (using replay library)` to replay multiple order books at the same time into the strategy bin

## Usage

Record (binary):

```bash
cd record
cargo run --release ../data/
```

Interleave (binary):

```bash
cd interleave
cargo run --release data/
```

Replay:

```bash
cargo run --release --manifest-path ./interleave/Cargo.toml data/ \
| cargo run --release --manifest-path ./strategy/Cargo.toml
```
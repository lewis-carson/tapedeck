# tapedeck

A toolkit to record and replay millisecond-level Binance order book data.

tapedeck is built to be modular - there are three ways to interact with the tools provided.

- `record`: records timestamped order book data to data/ directory. Serialised using json to be easily read by other tools
- `interleave`: takes n order book data files and interleaves them based on timestamp. Useful combining data from multiple ticker symbols
- `replay`: deserialises the data stream, builds a full order book from the partial updates and replays the order book back (in the same order as the original data)

For example, you could:
- `record` to record order book data into a directory
- `record > interleave` to playback multiple order books at the same time to stdout
- `record > interleave > strategy (using replay library)` to replay multiple interleaved order books at the same time

Notes on recording:
- Symbol minutes per full book (Symbolm/fb) is a useful metric: how often can is the average order book corrected with the full order book (building a local order book from partial updates gets out of sync)
- You can adjust N_SYMBOLS, CORRECTION_INTERVAL, and CORRECTION_TIMEOUT to change this metric
  - N_SYMBOLS - the number of symbols to record at the same time
  - CORRECTION_INTERVAL - how many ticks to correct the local order book with the full order book
  - CORRECTION_TIMEOUT - how often to read from the queue
- Higher is usually better, I like to aim for about 1.0 (1 full order book correction per minute)
- Sometimes the websocket callback dies because it's not responding fast enough

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
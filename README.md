# tapedeck

A toolkit to record and replay millisecond-level Binance order book data.

tapedeck is built to be modular - there are three ways to interact with the tools provided.

- `record`: records timestamped order book deata to data/ directory. Serialised using json to be easily read by other tools
- `interleave`: takes n order book data files and interleaves them based on timestamp. Useful combining data from multiple sources or multiple ticker symbols
- `replay`: deserialises the data stream, builds a full order book from the partial updates and replays the order book at the recorded timestamps

For example, you could:
- `record` order book data for a specific ticker symbol for use in a different tool
- `record > interleave` to playback multiple order books at the same time
- `record > interleave > replay` to replay multiple order books at the same time into a backtesting tool
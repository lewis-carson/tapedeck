# tapedeck

I first started developing tapedeck due to the lack of options for recording high quality order book data from cryptocurrency exchanges. I wanted to record order book data in a way that was easy to replay and analyse. I also wanted to be able to record data from multiple symbols at the same time and replay them in sync. Right now only Binance is supported but other exchanges arae on the way.

I started developing tapedeck - it's built to be modular, and each tool "snaps" together to allow you to form pipelines of data processing.

The entire thing is built on Unix pipes. While this might seem like a strange choice, it's makes for pretty reliable transport of event streams.

tapdeck uses `just` to manage how its commands are run.

Here's a quickstart guide on some of the things you might want to do with tapedeck:

Record data from Binance:
```bash
just record data/
```

Play back + accumulate full order books:
```bash
interleave data/ | accumulate
```

Stream live data from `server` + accumulate full order books:
```bash
ssh server -t 'tail -fq /path/to/data/*' | accumulate
```


Stream live data from `server` + accumulate full order books + extract symbols and midpoints:
```bash
ssh server 'tail -fq /path/to/data/*'
| accumulate
| jq '{
    symbol: .symbol,
    midpoint: (
    (
        (.event.FullOrderBook.asks[0].price | tonumber) +
        (.event.FullOrderBook.bids[0].price | tonumber)
    )/2
    )
}'
```

Spin up a terminal dashboard which listens to an event stream and displays some key information:

```bash
just run watch data/*
```

## Other Notes

Notes on recording:
- Symbol minutes per full book (Symbolm/fb) is a useful metric: how often can is the average order book corrected with the full order book (building a local order book from partial updates gets out of sync)
- You can adjust N_SYMBOLS, CORRECTION_INTERVAL, and CORRECTION_TIMEOUT to change this metric
  - N_SYMBOLS - the number of symbols to record at the same time
  - CORRECTION_INTERVAL - how many ticks to correct the local order book with the full order book
  - CORRECTION_TIMEOUT - how often to read from the queue
- Higher is usually better, I like to aim for about 1.0 (1 full order book correction per minute)
- Sometimes the websocket callback dies because it's not responding fast enough

Other notes:
- -Zon-broken-pipe=kill is amazing. When the previous process closes the pipe, the next process will too. Cool.
  - This allows us to consume some of the data and stop when we want to.
  - You can think of this as data flowing forwards (record > interleave > accumulate) but halting flowing backwards (accumulate > interleave > record)
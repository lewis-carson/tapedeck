interleave data2/ |
accumulate |
jq '{
    symbol: .symbol,
    midpoint: (
    (
        (.event.FullOrderBook.asks[0].price | tonumber) +
        (.event.FullOrderBook.bids[0].price | tonumber)
    )/2
    ),
    spread: (
        (.event.FullOrderBook.asks[0].price | tonumber) -
        (.event.FullOrderBook.bids[0].price | tonumber)
    ),
    spread_bps: (
        (
            (
                (.event.FullOrderBook.asks[0].price | tonumber) -
                (.event.FullOrderBook.bids[0].price | tonumber)
            ) /
            (
                (
                    (.event.FullOrderBook.asks[0].price | tonumber) +
                    (.event.FullOrderBook.bids[0].price | tonumber)
                ) / 2
            )
        ) * 10000
    )
}' |
jq 'select(.spread_bps > 10)'


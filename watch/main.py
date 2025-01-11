# iterate through stdin

import sys
from rich.console import Console
from rich.layout import Layout
import json
import os
from rich.console import Console, ConsoleOptions, RenderResult
from collections import defaultdict
from rich.panel import Panel


class ScrollToBottom:
    def __init__(self, text):
        self.text = text

    def __rich_console__(
        self, console: Console, options: ConsoleOptions
    ) -> RenderResult:
        height = options.max_height
        yield "\n".join(self.text[-height:])


console = Console()
layout = Layout()

# Divide the "screen" in to three parts
layout.split_column(
    Layout(name="header", size=3),
    Layout(ratio=3, name="graph"),
    Layout(ratio=1, name="logs"),
)
# Divide the "main" layout in to "side" and "body"
layout["logs"].split_row(Layout(name="partials"), Layout(name="fulls"))


from rich.live import Live
from time import sleep

with Live(layout, screen=True) as live:
    fmts = {
        "PartialOrderBook": "{event[receive_time]:<20} {event[symbol]}",
        "FullOrderBook": "{event[receive_time]:<20} {event[symbol]:<10} {event[event][FullOrderBook][bids][0][price]:<15} {event[event][FullOrderBook][asks][0][price]:<15}",
    }

    histories = defaultdict(list)
    for line in sys.stdin:

        js = json.loads(line)

        symbol = js["symbol"]
        event_type = list(js["event"].keys())[0]

        fmt = fmts[event_type]
        formatted_event = fmt.format(event=js)
        histories[event_type].append(formatted_event)

        if len(histories[event_type]) > 100:
            histories[event_type].pop(0)

        layout["partials"].update(
            Panel(
                ScrollToBottom(histories["PartialOrderBook"]),
                title="Partial Order Books"
            )
        )
        layout["fulls"].update(
            Panel(ScrollToBottom(histories["FullOrderBook"]), title="Full Order Books")
        )

        live.refresh()

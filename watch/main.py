# iterate through stdin

import sys
from rich.console import Console
from rich.layout import Layout
import json
import os


console = Console()
layout = Layout()

# Divide the "screen" in to three parts
layout.split_column(
    Layout(name="header", size=3),
    Layout(ratio=1, name="main"),
    Layout(size=10, name="footer"),
)
# Divide the "main" layout in to "side" and "body"
layout["main"].split_row(
    Layout(name="side"),
    Layout(name="body", ratio=2)
)
# Divide the "side" layout in to two
layout["side"].split(Layout(), Layout())

def generate_format(ev_data):
    return str(ev_data)
    '''chat_completion = client.chat.completions.create(
        messages=[
            {
                "role": "user",
                "content": "Given an example event data (to be referenced as event), generate a format string. e.g. {'symbol':'BNBETH'} ---> '{event[symbol]}' be concise; each datapoint should fit on a single line use square brackets and single words instead of sentences you may assume the event data will be a dictionary do not use any information outside of the provided keys. The information provided is: " + str(ev_data),
            }
        ],
        model="gpt-4o",
    )
    return chat_completion.choices[0].message.content'''

def apply_format(ev_data, format):
    return format.format(event=ev_data)

from rich.live import Live
from time import sleep

with Live(layout, screen=True) as live:
    formats = {
        "PartialOrderBook": "{event}",
        "FullOrderBook": "{event}",
    }

    history = []
    for line in sys.stdin:
        history.append(line)

        js = json.loads(line)

        symbol = js["symbol"]
        event_type = list(js["event"].keys())[0]

        '''
        if event_type not in formats:
            formats[event_type] = generate_format(js)
        '''

        formatted_event = apply_format(js, formats[event_type])
        history.append(formatted_event)
        
        size = layout["body"].minimum_size

        layout["body"].update("\n".join(history[:-size]))


        
        live.refresh()
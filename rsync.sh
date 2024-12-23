#!/bin/bash

rsync -e "ssh -i ~/.ssh/rpikey" -avP raspberrypi.local:/home/lew/drive/lobs/data/ imported_data/
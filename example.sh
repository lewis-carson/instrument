#!/bin/bash
# Clock demonstration - seconds on main dial, minutes on complication
while true; do
    # Get current seconds (0-59) for main dial
    seconds=$(date +%S | sed 's/^0//')
    if [ -z "$seconds" ]; then
        seconds=0
    fi
    
    # Get current minutes (0-59) for complication
    minutes=$(date +%M | sed 's/^0//')
    if [ -z "$minutes" ]; then
        minutes=0
    fi
    
    # Get current time for readout display
    current_time=$(date +%H:%M:%S)
    
    # needle1: seconds (0-59) - big hand
    needle1=$seconds
    # needle2: minutes (0-59) - little hand on complication
    needle2=$minutes
    
    # readout: shows current seconds
    readout=$seconds
    
    echo "needle1=${needle1} needle2=${needle2} readout=${readout} needle1label=SEC needle2label=MIN"
    
    sleep 1
done | cargo run --release -- --title "Clock - Seconds/Minutes" --range 0 59 --
#!/bin/bash
# Sine wave demonstration
t=0
sp=40
while true; do
    # Calculate sine wave values
    # needle1: main sine wave (0-100 range)
    needle1=$(echo "scale=2; 49 + $sp * s($t)" | bc -l)
    
    # readout: shows the primary needle value
    readout=$(echo "scale=1; $needle1" | bc -l)
    
    echo "needle1=${needle1} readout=${readout}"
    
    # Increment time for smooth animation
    t=$(echo "scale=4; $t + 0.05" | bc -l)
    
    sleep 0.5
done | cargo run --release -- --title "Sine Wave Demo" --range 0 100 --highlight 20 30 --
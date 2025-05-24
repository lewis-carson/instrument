#!/bin/bash
while true; do
    # Get the RAM usage percentage of the system
    ram_usage=$(vm_stat | awk '/Pages active/ {active=$3} /Pages speculative/ {speculative=$3} /Pages wired down/ {wired=$4} /Pages free/ {free=$3} END {used=(active+speculative+wired)*4096; total=(active+speculative+wired+free)*4096; print (used/total)*100}')
    # Output the RAM usage as an integer
    echo ${ram_usage} 
    sleep 0.1
done | cargo run --release -- --title "RAM Usage" --range 0 100 --
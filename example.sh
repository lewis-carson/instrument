#!/bin/bash
# System monitoring demonstration - RAM on main dial, CPU on complication
while true; do
    # Get RAM usage percentage (main dial)
    ram_usage=$(vm_stat | awk '
        /Pages free/ { free = $3 }
        /Pages active/ { active = $3 }
        /Pages inactive/ { inactive = $3 }
        /Pages speculative/ { spec = $3 }
        /Pages wired/ { wired = $3 }
        END {
            # Remove commas and calculate
            gsub(/,/, "", free); gsub(/,/, "", active); 
            gsub(/,/, "", inactive); gsub(/,/, "", spec); gsub(/,/, "", wired);
            total_pages = free + active + inactive + spec + wired;
            used_pages = active + inactive + spec + wired;
            printf "%.1f", (used_pages / total_pages) * 100;
        }')
    
    # Get CPU usage percentage (complication)
    cpu_usage=$(ps -A -o %cpu | awk '{s+=$1} END {printf "%.1f", s}')
    
    # Clamp CPU usage to 0-100 range
    cpu_usage=$(echo "$cpu_usage" | awk '{if($1 > 100) print 100; else print $1}')
    
    # needle1: RAM usage (0-100%)
    needle1=$ram_usage
    # needle2: CPU usage (0-100%) 
    needle2=$cpu_usage
    
    # readout: shows the RAM usage percentage
    readout=$ram_usage
    
    echo "needle1=${needle1} needle2=${needle2} highlightlower=90 highlightupper=100 readout=${readout} needle1label=RAM needle2label=CPU"
    
    sleep 1
done | cargo run --release -- --title "System Monitor - RAM/CPU" --range 0 100 --
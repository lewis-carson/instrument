# Instrument

A real-time instrument dial visualization tool that displays needles and highlight bands based on piped input data.

I'm following the suckless convention of having a config.rs file and you'll have to recompile if you want anything different. Defaults are already pretty good. Anything worth dynamically changing is a flag. See example.

You might have to bring your own font. See config.rs.

## Command Line Options

- `--range <min> <max>` - Set the dial range (default: 0 100)
- `--title <title>` - Set the window title (default: "Instrument")  
- `--highlight <lower> <upper>` - Set static highlight bounds that override input data

## Input Formats

The program accepts input via stdin in two formats:

### Key-Value Format (Recommended)
Send space-separated key=value pairs on each line:
```
needle1=75 needle2=25 readout=50 highlightlower=20 highlightupper=80
```

**Supported Keys:**
- `needle1` - Primary needle value (black)
- `needle2` - Secondary needle value (blue) 
- `readout` - Numeric display value (shown as large text)
- `highlightlower` - Lower bound of highlight band
- `highlightupper` - Upper bound of highlight band

### Legacy Single Value Format
Send a single numeric value per line (backwards compatibility):
```
75.5
```
This sets both `needle1` and `readout` to the same value.

## Behavior

- Needles and highlight bounds smoothly interpolate to new target positions
- Values outside the dial range turn the display red and show a warning indicator
- Command-line `--highlight` overrides any `highlightlower`/`highlightupper` from input
- Missing keys in input data hide the corresponding elements
- All numeric values support floating point precision

## Example Usage

```bash
# Static highlight with sine wave data
./example.sh

# Dynamic highlight from input data  
echo "needle1=50 needle2=75 highlightlower=40 highlightupper=60" | ./instrument

# Single value mode
echo "42.5" | ./instrument --range 0 100
```

TODO:

- ~~Highlight range flag~~ ✓
- ~~Multiple needles~~ ✓  
- Needle head types
- Allow window resizing but keep aspect ratio
- Only redraw needle and readout
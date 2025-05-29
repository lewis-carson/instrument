use instrument::{Instrument, InstrumentConfig, InstrumentCommand};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rand::Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an instrument with all three needle types using the bon-generated builder
    let config = InstrumentConfig::builder()
        .minor_tick_length(15)
        .major_tick_length(15)
        // Enable chronograph as a separate complication
        .build();

    let mut instrument = Instrument::new(config);
    
    // Create a channel for sending random commands
    let (sender, receiver) = mpsc::channel();
    
    // Spawn a thread to generate random commands continuously
    thread::spawn(move || {
        let mut rng = rand::rng();
        loop {
            // Create all commands and send them
            let commands = [
                InstrumentCommand::SetAllNeedles(
                    rng.random_range(0.0..100.0),
                    rng.random_range(0.0..100.0), 
                    rng.random_range(0.0..60.0),
                    rng.random_range(0.0..100.0)
                ),
                InstrumentCommand::SetReadout(rng.random_range(0.0..100.0)),
                InstrumentCommand::SetHighlightBounds(
                    rng.random_range(10.0..40.0),
                    rng.random_range(60.0..90.0)
                ),
            ];
            
            // Send all commands, break if any fail
            if commands.iter().any(|cmd| sender.send(cmd.clone()).is_err()) {
                break;
            }
            
            thread::sleep(Duration::from_millis(100));
        }
    });
    
    println!("Displaying instrument with randomly moving needles:");
    println!("- Primary needle: randomly moving (main gauge)");
    println!("- Secondary needle: randomly moving (main gauge)"); 
    println!("- Chronograph needle: randomly moving (separate dial)");
    println!("- Secondary chronograph needle: randomly moving (separate dial)");
    println!("- Highlight bounds: randomly changing");
    println!("Press Ctrl+C to exit");
    
    // Show the instrument with the command stream
    instrument.show_with_commands(receiver)
}

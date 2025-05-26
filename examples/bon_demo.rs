use instrument::{Instrument, InstrumentConfig, Color};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an instrument using the bon-generated builder
    let config = InstrumentConfig::builder()
        .title("Bon Demo Gauge".to_string())
        .range((0.0, 100.0))
        .curved_text("BUILT WITH BON".to_string())
        .primary_label("Percentage".to_string())
        .highlight_band((80.0, 100.0, Color::new(255, 0, 0))) // Red zone at high values
        .background_color(Color::new(10, 10, 30))
        .text_color(Color::new(255, 255, 255))
        .build();

    let mut instrument = Instrument::new(config);
    
    // Set a value and show the instrument
    instrument.set_value(75.0);
    instrument.show()
}

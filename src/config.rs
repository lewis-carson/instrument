pub const NUM_TICKS: usize = 11; // Number of major ticks
pub const MINOR_TICKS_PER_INTERVAL: usize = 5; // Number of minor ticks between each major tick
pub const TICK_LENGTH: i32 = 50; // Length of major ticks
pub const MINOR_TICK_LENGTH: i32 = 35; // Length of minor ticks
pub const TICK_THICKNESS: f32 = 2.0; // Thickness of major ticks
pub const MINOR_TICK_THICKNESS: f32 = 0.5; // Thickness of minor ticks

pub const DIAL_MARGIN: i32 = 64; // Margin around the dial
pub const DIAL_THICKNESS: i32 = 4; // Thickness of the dial arc

pub const NEEDLE_LENGTH_FACTOR: f64 = 1.1; // Factor to scale the needle length relative to the dial radius
pub const NEEDLE_BACK_LENGTH: f64 = 80.0; // Length of the needle extending behind the center
pub const NEEDLE_CROSSBAR_LENGTH: f64 = 24.0; // Length of the crossbar at the needle base
pub const NEEDLE_CROSSBAR_THICKNESS: f32 = 4.0; // Thickness of the crossbar at the needle base
pub const NEEDLE_WIDTH: f32 = 4.0; // Width of the needle

pub const READOUT_X_FACTOR: f64 = 0.72; // X position factor for the readout (relative to width)
pub const READOUT_Y_FACTOR: f64 = 0.80; // Y position factor for the readout (relative to height)
pub const READOUT_BIG_FONT_SIZE: f32 = 54.0; // Font size for the integer part of the readout
pub const READOUT_SMALL_FONT_SIZE: f32 = 28.0; // Font size for the fractional part of the readout

pub const DIAL_NUMBERS_FONT_SIZE: f32 = 30.0; // Font size for the dial numbers
pub const TICKS_TO_NUMBERS_DISTANCE: f64 = 30.0; // Distance between the ticks and the numbers on the dial

pub const WINDOW_WIDTH: usize = 350; // Width of the application window
pub const WINDOW_HEIGHT: usize = 350; // Height of the application window

pub const FONT_DATA: &[u8] = include_bytes!("BerkeleyMono-Regular.otf"); // Include the font file at compile time
pub const EXCLAMATION_MARK_FONT_SIZE: f32 = 50.0; // Font size for the exclamation mark

pub enum CrossbarType {
    DOT, // A circle drawn instead of the bar
    BAR, // The default bar type
}

pub const DEFAULT_CROSSBAR_TYPE: CrossbarType = CrossbarType::DOT; // Default crossbar type
pub const DOT_RADIUS: i32 = 8; // Radius of the dot for the DOT crossbar type

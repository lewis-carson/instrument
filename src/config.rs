pub const NUM_TICKS: usize = 11; // Number of major ticks
pub const MINOR_TICKS_PER_INTERVAL: usize = 5; // Number of minor ticks between each major tick
pub const TICK_LENGTH: i32 = 40; // Length of major ticks
pub const MINOR_TICK_LENGTH: i32 = 25; // Length of minor ticks
pub const TICK_THICKNESS: f32 = 2.0; // Thickness of major ticks
pub const MINOR_TICK_THICKNESS: f32 = 0.5; // Thickness of minor ticks

pub const DIAL_MARGIN: i32 = 32; // Margin around the dial
pub const DIAL_THICKNESS: i32 = 4; // Thickness of the dial arc
pub const DIAL_NUMBERS_FONT_SIZE: f32 = 30.0; // Font size for the dial numbers

pub const TICKS_TO_NUMBERS_DISTANCE: f64 = 30.0; // Distance between the ticks and the numbers on the dial

pub const NEEDLE_LENGTH_FACTOR: f64 = 1.05; // Factor to scale the needle length relative to the dial radius
pub const NEEDLE_BACK_LENGTH: f64 = 80.0; // Length of the needle extending behind the center
pub const NEEDLE_WIDTH: f32 = 4.0; // Width of the needle

pub const LERP_FACTOR: f64 = 0.1; // Speed of interpolation for smooth animations (0.0 = no movement, 1.0 = instant)

pub const READOUT_X_FACTOR: f64 = 0.69; // X position factor for the readout (relative to width)
pub const READOUT_Y_FACTOR: f64 = 0.75; // Y position factor for the readout (relative to height)
pub const READOUT_BIG_FONT_SIZE: f32 = 54.0; // Font size for the integer part of the readout
pub const READOUT_SMALL_FONT_SIZE: f32 = 28.0; // Font size for the fractional part of the readout

pub const READOUT_BOX_PADDING: i32 = 30; // Padding around the readout box
pub const READOUT_BOX_THICKNESS: f32 = 4.0; // Thickness of the readout box lines

pub const WINDOW_WIDTH: usize = 350; // Width of the application window
pub const WINDOW_HEIGHT: usize = 350; // Height of the application window
pub const MAX_FRAMERATE: f64 = 60.0; // Maximum framerate (frames per second)

pub const FONT_DATA: &[u8] = include_bytes!("BerkeleyMono-Regular.otf"); // Include the font file at compile time
pub const EXCLAMATION_MARK_FONT_SIZE: f32 = 50.0; // Font size for the exclamation mark

pub const DOT_RADIUS: i32 = 6; // Radius of the dot for the DOT crossbar type

// Highlight band configuration
pub const HIGHLIGHT_BAND_WIDTH: i32 = 35; // Width of the highlight band
pub const HIGHLIGHT_COLOR: (u8, u8, u8) = (0xff, 0x00, 0x00); // Red color for the highlight band
pub const HIGHLIGHT_ALPHA: f64 = 1.0; // Transparency for the highlight band
pub const HIGHLIGHT_EDGE_SOFTNESS: f64 = 0.005; // Angular threshold for highlight edge anti-aliasing (in radians)

// Needle2 complication configuration
pub const NEEDLE2_USE_COMPLICATION: bool = true; // Move needle2 to a smaller dial in the top middle like a watch complication
pub const COMPLICATION_SHIFT: i32 = 130; // Vertical offset for the complication dial position (positive moves down)
pub const COMPLICATION_SIZE: f64 = 7.0; // Size factor for the complication dial (radius = width.min(height) / COMPLICATION_SIZE)

// Mini dial (complication) specific configuration
pub mod mini_dial {
    pub const NUM_TICKS: usize = 5; // Number of major ticks
    pub const MINOR_TICKS_PER_INTERVAL: usize = 0; // Number of minor ticks between each major tick
    pub const TICK_LENGTH: i32 = 10; // Length of major ticks
    pub const MINOR_TICK_LENGTH: i32 = 4; // Length of minor ticks
    pub const TICK_THICKNESS: f32 = 2.0; // Thickness of major ticks
    pub const MINOR_TICK_THICKNESS: f32 = 0.5; // Thickness of minor ticks

    pub const DIAL_MARGIN: i32 = 10; // Margin around the dial
    pub const DIAL_THICKNESS: i32 = 2; // Thickness of the dial arc

    pub const NEEDLE_LENGTH_FACTOR: f64 = 1.0; // Factor to scale the needle length relative to the dial radius
    pub const NEEDLE_BACK_LENGTH: f64 = 30.0; // Length of the needle extending behind the center
    pub const NEEDLE_WIDTH: f32 = 4.0; // Width of the needle

    pub const DIAL_NUMBERS_FONT_SIZE: f32 = 30.0; // Font size for the dial numbers
    pub const TICKS_TO_NUMBERS_DISTANCE: f64 = 30.0; // Distance between the ticks and the numbers on the dial

    pub const DOT_RADIUS: i32 = 8; // Radius of the dot for the DOT crossbar type
}

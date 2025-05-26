/// Color representation for gauge elements
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    pub const fn as_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Configuration for tick marks on the dial
#[derive(Debug, Clone)]
pub struct TickConfig {
    pub num_ticks: usize,
    pub minor_ticks_per_interval: usize,
    pub major_length: i32,
    pub minor_length: i32,
    pub major_thickness: f32,
    pub minor_thickness: f32,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            num_ticks: 11,
            minor_ticks_per_interval: 5,
            major_length: 40,
            minor_length: 25,
            major_thickness: 2.0,
            minor_thickness: 0.5,
        }
    }
}

/// Configuration for the main dial appearance
#[derive(Debug, Clone)]
pub struct DialConfig {
    pub margin: i32,
    pub thickness: i32,
    pub numbers_font_size: f32,
    pub ticks_to_numbers_distance: f64,
}

impl Default for DialConfig {
    fn default() -> Self {
        Self {
            margin: 45,
            thickness: 4,
            numbers_font_size: 30.0,
            ticks_to_numbers_distance: 30.0,
        }
    }
}

/// Configuration for needle appearance and behavior
#[derive(Debug, Clone)]
pub struct NeedleConfig {
    pub length_factor: f64,
    pub back_length: f64,
    pub width: f32,
    pub lerp_factor: f64,
}

impl Default for NeedleConfig {
    fn default() -> Self {
        Self {
            length_factor: 1.05,
            back_length: 80.0,
            width: 4.0,
            lerp_factor: 0.1,
        }
    }
}

/// Configuration for the digital readout display
#[derive(Debug, Clone)]
pub struct ReadoutConfig {
    pub x_factor: f64,
    pub y_factor: f64,
    pub big_font_size: f32,
    pub small_font_size: f32,
    pub box_padding: i32,
    pub box_thickness: f32,
}

impl Default for ReadoutConfig {
    fn default() -> Self {
        Self {
            x_factor: 0.69,
            y_factor: 0.75,
            big_font_size: 54.0,
            small_font_size: 28.0,
            box_padding: 30,
            box_thickness: 4.0,
        }
    }
}

/// Configuration for application window
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub width: usize,
    pub height: usize,
    pub max_framerate: f64,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 300,
            height: 300,
            max_framerate: 60.0,
        }
    }
}

/// Configuration for fonts and text rendering
#[derive(Debug, Clone)]
pub struct FontConfig {
    pub data: &'static [u8],
    pub exclamation_mark_size: f32,
    pub dot_radius: i32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            data: include_bytes!("BerkeleyMono-Regular.otf"),
            exclamation_mark_size: 50.0,
            dot_radius: 6,
        }
    }
}

/// Configuration for curved text display
#[derive(Debug, Clone)]
pub struct CurvedTextConfig {
    pub text: &'static str,
    pub font_size: f32,
    pub radius_offset: f64,
    pub arc_span: f64,
    pub angle: f64,
}

impl Default for CurvedTextConfig {
    fn default() -> Self {
        Self {
            text: "INSTRUMENT GAUGE",
            font_size: 30.0,  // Reduced from 60.0 for better rendering
            radius_offset: 15.0,
            arc_span: std::f64::consts::PI * 0.23,
            angle: 3.0 * std::f64::consts::PI / 2.0,
        }
    }
}

/// Configuration for highlight band
#[derive(Debug, Clone)]
pub struct HighlightBandConfig {
    pub width: i32,
    pub color: Color,
    pub alpha: f64,
    pub edge_softness: f64,
}

impl Default for HighlightBandConfig {
    fn default() -> Self {
        Self {
            width: 35,
            color: Color::new(0xff, 0x00, 0x00),
            alpha: 1.0,
            edge_softness: 0.005,
        }
    }
}

/// Configuration for needle2 complication (mini dial)
#[derive(Debug, Clone)]
pub struct ComplicationConfig {
    pub use_complication: bool,
    pub shift: i32,
    pub size: f64,
}

impl Default for ComplicationConfig {
    fn default() -> Self {
        Self {
            use_complication: true,
            shift: 130,
            size: 7.0,
        }
    }
}

/// Main configuration struct containing all gauge settings
#[derive(Debug, Clone)]
pub struct Config {
    pub ticks: TickConfig,
    pub dial: DialConfig,
    pub needle: NeedleConfig,
    pub readout: ReadoutConfig,
    pub window: WindowConfig,
    pub font: FontConfig,
    pub curved_text: CurvedTextConfig,
    pub highlight_band: HighlightBandConfig,
    pub complication: ComplicationConfig,
    pub mini_dial: mini_dial::MiniDialFullConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ticks: TickConfig::default(),
            dial: DialConfig::default(),
            needle: NeedleConfig::default(),
            readout: ReadoutConfig::default(),
            window: WindowConfig::default(),
            font: FontConfig::default(),
            curved_text: CurvedTextConfig::default(),
            highlight_band: HighlightBandConfig::default(),
            complication: ComplicationConfig::default(),
            mini_dial: mini_dial::MiniDialFullConfig::default(),
        }
    }
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }
}

/// Global configuration instance
pub static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| Config::new());

/// Mini dial (complication) configuration module
pub mod mini_dial {

    /// Configuration for mini dial ticks
    #[derive(Debug, Clone)]
    pub struct MiniTickConfig {
        pub num_ticks: usize,
        pub minor_ticks_per_interval: usize,
        pub major_length: i32,
        pub minor_length: i32,
        pub major_thickness: f32,
        pub minor_thickness: f32,
    }

    impl MiniTickConfig {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for MiniTickConfig {
        fn default() -> Self {
            Self {
                num_ticks: 5,
                minor_ticks_per_interval: 0,
                major_length: 10,
                minor_length: 4,
                major_thickness: 2.0,
                minor_thickness: 0.5,
            }
        }
    }

    /// Configuration for mini dial appearance
    #[derive(Debug, Clone)]
    pub struct MiniDialConfig {
        pub margin: i32,
        pub thickness: i32,
        pub numbers_font_size: f32,
        pub ticks_to_numbers_distance: f64,
        pub dot_radius: i32,
    }

    impl MiniDialConfig {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for MiniDialConfig {
        fn default() -> Self {
            Self {
                margin: 15,
                thickness: 2,
                numbers_font_size: 30.0,
                ticks_to_numbers_distance: 30.0,
                dot_radius: 8,
            }
        }
    }

    /// Configuration for mini dial needle
    #[derive(Debug, Clone)]
    pub struct MiniNeedleConfig {
        pub length_factor: f64,
        pub back_length: f64,
        pub width: f32,
    }

    impl MiniNeedleConfig {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for MiniNeedleConfig {
        fn default() -> Self {
            Self {
                length_factor: 1.0,
                back_length: 30.0,
                width: 4.0,
            }
        }
    }

    /// Complete mini dial configuration
    #[derive(Debug, Clone)]
    pub struct MiniDialFullConfig {
        pub ticks: MiniTickConfig,
        pub dial: MiniDialConfig,
        pub needle: MiniNeedleConfig,
    }

    impl MiniDialFullConfig {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for MiniDialFullConfig {
        fn default() -> Self {
            Self {
                ticks: MiniTickConfig::default(),
                dial: MiniDialConfig::default(),
                needle: MiniNeedleConfig::default(),
            }
        }
    }
}

// ============================================================================
// CRATE CONFIGURATION & IMPORTS
// ============================================================================

// External crate imports
use bon::Builder;
use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use rusttype::{Font, Scale};

// Standard library imports
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::time::Instant;

// Window management imports
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

// ============================================================================
// COLOR CONFIGURATION
// ============================================================================

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

// ============================================================================
// PUBLIC API - MAIN INTERFACE
// ============================================================================

/// Command enum for type-safe instrument updates
#[derive(Debug, Clone)]
pub enum InstrumentCommand {
    SetPrimaryNeedle(f64),
    SetSecondaryNeedle(f64),
    SetReadout(f64),
    SetReadoutText(String),
    SetHighlightBounds(f64, f64),
    SetBothNeedles(f64, f64), // primary, secondary
}

/// Main instrument struct - the primary public interface
#[derive(Debug, Clone)]
pub struct Instrument {
    config: InstrumentConfig,
    state: InstrumentState,
}

#[derive(Debug, Clone, Builder)]
pub struct InstrumentConfig {
    #[builder(default = "Instrument".to_string())]
    pub title: String,
    #[builder(default = (0.0, 100.0))]
    pub range: (f64, f64),
    pub highlight_band: Option<(f64, f64, Color)>,
    #[builder(default = false)]
    pub dual_needle: bool,
    
    // Window configuration
    #[builder(default = 300)]
    pub window_width: usize,
    #[builder(default = 300)]
    pub window_height: usize,
    #[builder(default = 60.0)]
    pub max_framerate: f64,
    
    // Main dial configuration
    #[builder(default = 45)]
    pub dial_margin: i32,
    #[builder(default = 4)]
    pub dial_thickness: i32,
    #[builder(default = 30.0)]
    pub dial_numbers_font_size: f32,
    #[builder(default = 30.0)]
    pub dial_ticks_to_numbers_distance: f64,
    
    // Tick configuration
    #[builder(default = 11)]
    pub ticks_count: usize,
    #[builder(default = 5)]
    pub minor_ticks_per_interval: usize,
    #[builder(default = 40)]
    pub major_tick_length: i32,
    #[builder(default = 25)]
    pub minor_tick_length: i32,
    #[builder(default = 2.0)]
    pub major_tick_thickness: f32,
    #[builder(default = 0.5)]
    pub minor_tick_thickness: f32,
    
    // Needle configuration
    #[builder(default = 1.05)]
    pub needle_length_factor: f64,
    #[builder(default = 80.0)]
    pub needle_back_length: f64,
    #[builder(default = 4.0)]
    pub needle_width: f32,
    #[builder(default = 0.1)]
    pub needle_lerp_factor: f64,
    
    // Secondary needle configuration
    #[builder(default = true)]
    pub use_secondary_complication: bool,
    #[builder(default = 130)]
    pub secondary_dial_shift: i32,
    #[builder(default = 7.0)]
    pub secondary_dial_size: f64,
    #[builder(default = 5)]
    pub secondary_ticks_count: usize,
    #[builder(default = 10)]
    pub secondary_tick_length: i32,
    #[builder(default = 15)]
    pub secondary_dial_margin: i32,
    #[builder(default = 2)]
    pub secondary_dial_thickness: i32,
    #[builder(default = 1.0)]
    pub secondary_needle_length_factor: f64,
    #[builder(default = 4.0)]
    pub secondary_needle_width: f32,
    #[builder(default = 30.0)]
    pub secondary_needle_back_length: f64,
    #[builder(default = 30.0)]
    pub secondary_dial_numbers_font_size: f32,
    #[builder(default = 30.0)]
    pub secondary_dial_ticks_to_numbers_distance: f64,
    #[builder(default = 8)]
    pub secondary_dial_dot_radius: i32,
    #[builder(default = 0)]
    pub secondary_minor_ticks_per_interval: usize,
    #[builder(default = 4)]
    pub secondary_minor_tick_length: i32,
    #[builder(default = 2.0)]
    pub secondary_major_tick_thickness: f32,
    #[builder(default = 0.5)]
    pub secondary_minor_tick_thickness: f32,
    
    // Readout configuration
    #[builder(default = 0.69)]
    pub readout_x_factor: f64,
    #[builder(default = 0.75)]
    pub readout_y_factor: f64,
    #[builder(default = 54.0)]
    pub readout_big_font_size: f32,
    #[builder(default = 28.0)]
    pub readout_small_font_size: f32,
    #[builder(default = 30)]
    pub readout_box_padding: i32,
    #[builder(default = 4.0)]
    pub readout_box_thickness: f32,
    
    // Curved text configuration
    #[builder(default = "INSTRUMENT GAUGE".to_string())]
    pub curved_text: String,
    #[builder(default = 30.0)]
    pub curved_text_font_size: f32,
    #[builder(default = 15.0)]
    pub curved_text_radius_offset: f64,
    #[builder(default = std::f64::consts::PI * 0.23)]
    pub curved_text_arc_span: f64,
    #[builder(default = 3.0 * std::f64::consts::PI / 2.0)]
    pub curved_text_angle: f64,
    
    // Labels
    #[builder(default = "Primary".to_string())]
    pub primary_label: String,
    #[builder(default = "Secondary".to_string())]
    pub secondary_label: String,
    
    // Highlight band configuration
    #[builder(default = 35)]
    pub highlight_band_width: i32,
    #[builder(default = Color::new(0xff, 0x00, 0x00))]
    pub highlight_band_color: Color,
    #[builder(default = 1.0)]
    pub highlight_band_alpha: f64,
    #[builder(default = 0.005)]
    pub highlight_band_edge_softness: f64,
    
    // Colors
    pub background_color: Option<Color>,
    pub text_color: Option<Color>,
    pub needle_color: Option<Color>,
    pub dial_color: Option<Color>,
    
    // Font configuration
    #[builder(default = include_bytes!("BerkeleyMono-Regular.otf"))]
    pub font_data: &'static [u8],
    #[builder(default = 50.0)]
    pub exclamation_mark_size: f32,
    #[builder(default = 6)]
    pub dot_radius: i32,
}

// ============================================================================
// CONFIGURATION TYPES (INTERNAL)
// ============================================================================

#[derive(Debug, Clone)]
struct InstrumentState {
    primary_value: f64,
    secondary_value: Option<f64>,
    readout: Option<String>,
    current_values: HashMap<String, f64>,
}

impl Instrument {
    pub fn set_value(&mut self, value: f64) {
        self.state.primary_value = value.clamp(self.config.range.0, self.config.range.1);
        self.state.current_values.insert("needle1".to_string(), self.state.primary_value);
        self.state.current_values.insert("readout".to_string(), self.state.primary_value);
    }

    pub fn set_primary_value(&mut self, value: f64) {
        self.state.primary_value = value.clamp(self.config.range.0, self.config.range.1);
        self.state.current_values.insert("needle1".to_string(), self.state.primary_value);
    }

    pub fn set_secondary_value(&mut self, value: f64) {
        if self.config.dual_needle {
            let clamped_value = value.clamp(self.config.range.0, self.config.range.1);
            self.state.secondary_value = Some(clamped_value);
            self.state.current_values.insert("needle2".to_string(), clamped_value);
        }
    }

    pub fn set_readout(&mut self, text: &str) {
        self.state.readout = Some(text.to_string());
    }

    pub fn show(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let title = self.config.title.clone();
        let range = self.config.range;
        let highlight_range = self.config.highlight_band.map(|(min, max, _color)| (min, max));
        
        // Create a channel for sending updates to the rendering thread
        let (sender, receiver) = mpsc::channel();
        
        // Send initial state
        sender.send(self.state.current_values.clone())?;
        
        // Store sender for future updates
        // Note: In a real implementation, we'd need to handle this better
        // This is a simplified example
        
        self.run_window(title, range, highlight_range, receiver)
    }

    pub fn show_with_receiver(&mut self, receiver: Receiver<HashMap<String, f64>>) -> Result<(), Box<dyn std::error::Error>> {
        let title = self.config.title.clone();
        let range = self.config.range;
        let highlight_range = self.config.highlight_band.map(|(min, max, _color)| (min, max));
        
        self.run_window(title, range, highlight_range, receiver)
    }

    pub fn show_with_commands(&mut self, receiver: Receiver<InstrumentCommand>) -> Result<(), Box<dyn std::error::Error>> {
        let title = self.config.title.clone();
        let range = self.config.range;
        let highlight_range = self.config.highlight_band.map(|(min, max, _color)| (min, max));
        
        self.run_window_with_commands(title, range, highlight_range, receiver)
    }

    fn run_window(
        &self,
        title: String,
        range: (f64, f64),
        highlight_range: Option<(f64, f64)>,
        receiver: Receiver<HashMap<String, f64>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This contains the main window logic from the original main.rs
        // Adapted to work with the library API
        
        let logical_width: usize = self.config.window_width;
        let logical_height: usize = self.config.window_height;
        
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(LogicalSize::new            (logical_width as f64, logical_height as f64))
            .with_resizable(false)
            .build(&event_loop)?;
        
        let window = std::sync::Arc::new(window);
        
        let mut app_state = AppState::new(range.0, range.1);
        if let Some((lower, upper)) = highlight_range {
            app_state.set_highlight_override(lower, upper);
        }
        
        let window_clone = window.clone();
        let size = window.inner_size();
        let mut fb_width = size.width as usize;
        let mut fb_height = size.height as usize;
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let mut pixels = Pixels::new(size.width, size.height, surface_texture)?;
        
        let target_fps = self.config.max_framerate;
        let frame_duration = std::time::Duration::from_secs_f64(1.0 / target_fps);
        let mut last_frame = Instant::now();

        event_loop.run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    },
                    WindowEvent::Resized(new_size) => {
                        fb_width = new_size.width as usize;
                        fb_height = new_size.height as usize;
                        let _ = pixels.resize_buffer(new_size.width, new_size.height);
                        let _ = pixels.resize_surface(new_size.width, new_size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        app_state.update(&receiver);
                        
                        let frame = pixels.frame_mut();
                        let mut canvas = Canvas::new(frame, fb_width, fb_height);
                        render_instrument(&mut canvas, &app_state, &self.config);
                        let _ = pixels.render();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if last_frame.elapsed() >= frame_duration {
                        window_clone.request_redraw();
                        last_frame = Instant::now();
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }

    fn run_window_with_commands(
        &self,
        title: String,
        range: (f64, f64),
        highlight_range: Option<(f64, f64)>,
        receiver: Receiver<InstrumentCommand>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let logical_width: usize = self.config.window_width;
        let logical_height: usize = self.config.window_height;
        
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(LogicalSize::new(logical_width as f64, logical_height as f64))
            .with_resizable(false)
            .build(&event_loop)?;
        
        let window = std::sync::Arc::new(window);
        
        let mut app_state = AppState::new(range.0, range.1);
        if let Some((lower, upper)) = highlight_range {
            app_state.set_highlight_override(lower, upper);
        }
        
        let window_clone = window.clone();
        let size = window.inner_size();
        let mut fb_width = size.width as usize;
        let mut fb_height = size.height as usize;
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let mut pixels = Pixels::new(size.width, size.height, surface_texture)?;
        
        let target_fps = self.config.max_framerate;
        let frame_duration = std::time::Duration::from_secs_f64(1.0 / target_fps);
        let mut last_frame = Instant::now();

        event_loop.run(move |event, window_target| {
            window_target.set_control_flow(ControlFlow::Poll);
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    },
                    WindowEvent::Resized(new_size) => {
                        fb_width = new_size.width as usize;
                        fb_height = new_size.height as usize;
                        let _ = pixels.resize_buffer(new_size.width, new_size.height);
                        let _ = pixels.resize_surface(new_size.width, new_size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        app_state.update_with_commands(&receiver);
                        
                        let frame = pixels.frame_mut();
                        let mut canvas = Canvas::new(frame, fb_width, fb_height);
                        render_instrument(&mut canvas, &app_state, &self.config);
                        let _ = pixels.render();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if last_frame.elapsed() >= frame_duration {
                        window_clone.request_redraw();
                        last_frame = Instant::now();
                    }
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }


}



impl Instrument {
    pub fn new(config: InstrumentConfig) -> Self {
        let state = InstrumentState {
            primary_value: config.range.0,
            secondary_value: if config.dual_needle { Some(config.range.0) } else { None },
            readout: None,
            current_values: HashMap::new(),
        };

        Self {
            config,
            state,
        }
    }
}

// ============================================================================
// INTERNAL IMPLEMENTATION (from main.rs)
// ============================================================================

// ============================================================================
// RETAINED MODE ABSTRACTIONS
// ============================================================================

#[derive(Clone, Debug)]
enum DrawCommand {
    Clear((u8, u8, u8)),
    Arc {
        cx: i32, cy: i32, r: i32, thickness: i32,
        start_angle: f64, arc_span: f64,
        color: (u8, u8, u8),
    },
    HighlightBand {
        cx: i32, cy: i32, r: i32,
        start_angle: f64, end_angle: f64,
        inner_radius: f64, outer_radius: f64,
    },
    Tick {
        cx: i32, cy: i32, r: i32,
        angle: f64, length: i32, thickness: f32,
        color: (u8, u8, u8),
    },
    Text {
        x: i32, y: i32, text: String,
        font_size: f32, color: (u8, u8, u8),
    },
    CurvedText {
        cx: i32, cy: i32, radius: f64, text: String,
        font_size: f32, arc_span: f64, start_angle: f64,
        color: (u8, u8, u8),
    },
    NeedleLine {
        x0: i32, y0: i32, x1: i32, y1: i32,
        thickness: f32, tapered: bool,
        color: (u8, u8, u8),
    },
    Circle {
        cx: i32, cy: i32, radius: i32,
        color: (u8, u8, u8),
    },
}

struct Scene {
    commands: Vec<DrawCommand>,
}

impl Scene {
    fn new(_width: usize, _height: usize) -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    fn render(&self, canvas: &mut Canvas, config: &InstrumentConfig) {
        for command in &self.commands {
            match command {
                DrawCommand::Clear(color) => {
                    canvas.clear(*color);
                }
                DrawCommand::Arc { cx, cy, r, thickness, start_angle, arc_span, color } => {
                    render_arc_immediate(canvas, *cx, *cy, *r, *thickness, *start_angle, *arc_span, *color);
                }
                DrawCommand::HighlightBand { cx, cy, r, start_angle, end_angle, inner_radius, outer_radius } => {
                    render_highlight_band_immediate(canvas, *cx, *cy, *r, *start_angle, *end_angle, *inner_radius, *outer_radius, config);
                }
                DrawCommand::Tick { cx, cy, r, angle, length, thickness, color } => {
                    let outer_x = *cx as f64 + angle.cos() * (*r as f64 - 1.0);
                    let outer_y = *cy as f64 + angle.sin() * (*r as f64 - 1.0);
                    let inner_x = *cx as f64 + angle.cos() * (*r as f64 - *length as f64);
                    let inner_y = *cy as f64 + angle.sin() * (*r as f64 - *length as f64);
                    draw_thick_line_aa(
                        canvas.frame, canvas.width,
                        inner_x.round() as i32, inner_y.round() as i32,
                        outer_x.round() as i32, outer_y.round() as i32,
                        *thickness, color.0, color.1, color.2
                    );
                }
                DrawCommand::Text { x, y, text, font_size, color } => {
                    let font = Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
                    let scale = Scale::uniform(*font_size);
                    draw_text(canvas.frame, canvas.width, canvas.height, *x, *y, text, &font, scale, *color);
                }
                DrawCommand::CurvedText { cx, cy, radius, text, font_size, arc_span, start_angle, color } => {
                    let font = Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
                    let scale = Scale::uniform(*font_size);
                    draw_curved_text(canvas, *cx, *cy, *radius, text, &font, scale, *arc_span, *start_angle, *color);
                }
                DrawCommand::NeedleLine { x0, y0, x1, y1, thickness, tapered, color } => {
                    if *tapered {
                        draw_thick_line_tapered_aa(canvas.frame, canvas.width, *x0, *y0, *x1, *y1, *thickness, color.0, color.1, color.2);
                    } else {
                        draw_thick_line_aa(canvas.frame, canvas.width, *x0, *y0, *x1, *y1, *thickness, color.0, color.1, color.2);
                    }
                }
                DrawCommand::Circle { cx, cy, radius, color } => {
                    draw_circle(canvas.frame, canvas.width, *cx, *cy, *radius, color.0, color.1, color.2);
                }
            }
        }
    }
}

// ============================================================================
// CORE DATA TYPES
// ============================================================================

struct Canvas<'a> {
    frame: &'a mut [u8],
    width: usize,
    height: usize,
}

impl<'a> Canvas<'a> {
    fn new(frame: &'a mut [u8], width: usize, height: usize) -> Self {
        Self { frame, width, height }
    }

    fn clear(&mut self, color: (u8, u8, u8)) {
        for chunk in self.frame.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[color.0, color.1, color.2, 0xff]);
        }
    }
}

struct HighlightBounds {
    lower: f64,
    upper: f64,
    target_lower: f64,
    target_upper: f64,
}

impl HighlightBounds {
    fn new(lower: f64, upper: f64) -> Self {
        Self {
            lower,
            upper,
            target_lower: lower,
            target_upper: upper,
        }
    }

    fn set_target_bounds(&mut self, lower: f64, upper: f64) {
        self.target_lower = lower;
        self.target_upper = upper;
    }

    fn update_position(&mut self) {
        self.lower = lerp(self.lower, self.target_lower);
        self.upper = lerp(self.upper, self.target_upper);
    }

    fn get_bounds(&self) -> (f64, f64) {
        (self.lower, self.upper)
    }
}

struct AppState {
    needle1: Option<Needle>,
    needle2: Option<Needle>,
    current_values: HashMap<String, f64>,
    min_value: f64,
    max_value: f64,
    highlight_bounds: Option<HighlightBounds>,
    highlight_override: Option<(f64, f64)>, // Command-line override for highlight range
}

impl AppState {
    fn new(min_value: f64, max_value: f64) -> Self {
        Self {
            needle1: None,
            needle2: None,
            current_values: HashMap::new(),
            min_value,
            max_value,
            highlight_bounds: None,
            highlight_override: None,
        }
    }

    fn update(&mut self, receiver: &Receiver<HashMap<String, f64>>) {
        // Try to get the latest values without blocking
        while let Ok(values) = receiver.try_recv() {
            // Clear current values and update with new ones
            self.current_values.clear();
            self.current_values.extend(values.clone());
            
            // Update needle1
            if let Some(value) = values.get("needle1") {
                if self.needle1.is_none() {
                    self.needle1 = Some(Needle::new());
                }
                if let Some(ref mut needle) = self.needle1 {
                    let target_pos = ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                    needle.set_target_pos(target_pos);
                }
            } else {
                self.needle1 = None;
            }
            
            // Update needle2
            if let Some(value) = values.get("needle2") {
                if self.needle2.is_none() {
                    self.needle2 = Some(Needle::new());
                }
                if let Some(ref mut needle) = self.needle2 {
                    let target_pos = ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                    needle.set_target_pos(target_pos);
                }
            } else {
                self.needle2 = None;
            }
            
            // Update highlight range from highlightlower and highlightupper
            // But only if there's no command-line override
            if self.highlight_override.is_none() {
                if let (Some(&lower), Some(&upper)) = (values.get("highlightlower"), values.get("highlightupper")) {
                    let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
                    if let Some(ref mut bounds) = self.highlight_bounds {
                        bounds.set_target_bounds(min_bound, max_bound);
                    } else {
                        self.highlight_bounds = Some(HighlightBounds::new(min_bound, max_bound));
                    }
                } else {
                    self.highlight_bounds = None;
                }
            }
        }

        // Update needle positions
        if let Some(ref mut needle) = self.needle1 {
            needle.update_position();
        }
        if let Some(ref mut needle) = self.needle2 {
            needle.update_position();
        }
        
        // Update highlight bounds position
        if let Some(ref mut bounds) = self.highlight_bounds {
            bounds.update_position();
        }
        
        // If no values received yet, do random updates
        if self.current_values.is_empty() {
            if let Some(ref mut needle) = self.needle1 {
                needle.update_random();
            }
            if let Some(ref mut needle) = self.needle2 {
                needle.update_random();
            }
        }
    }

    fn update_with_commands(&mut self, receiver: &Receiver<InstrumentCommand>) {
        // Try to get the latest command without blocking
        while let Ok(command) = receiver.try_recv() {
            match command {
                InstrumentCommand::SetPrimaryNeedle(value) => {
                    self.current_values.insert("needle1".to_string(), value);
                    if self.needle1.is_none() {
                        self.needle1 = Some(Needle::new());
                    }
                    if let Some(ref mut needle) = self.needle1 {
                        let target_pos = ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                        needle.set_target_pos(target_pos);
                    }
                },
                InstrumentCommand::SetSecondaryNeedle(value) => {
                    self.current_values.insert("needle2".to_string(), value);
                    if self.needle2.is_none() {
                        self.needle2 = Some(Needle::new());
                    }
                    if let Some(ref mut needle) = self.needle2 {
                        let target_pos = ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                        needle.set_target_pos(target_pos);
                    }
                },
                InstrumentCommand::SetReadout(value) => {
                    self.current_values.insert("readout".to_string(), value);
                },
                InstrumentCommand::SetReadoutText(text) => {
                    // Store the text in a way that can be used by the rendering system
                    // For now, we'll just trigger an update (the actual text rendering would need more work)
                    self.current_values.insert("readout_text".to_string(), text.len() as f64);
                },
                InstrumentCommand::SetHighlightBounds(lower, upper) => {
                    self.current_values.insert("highlightlower".to_string(), lower);
                    self.current_values.insert("highlightupper".to_string(), upper);
                    
                    // Update highlight range if there's no command-line override
                    if self.highlight_override.is_none() {
                        let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
                        if let Some(ref mut bounds) = self.highlight_bounds {
                            bounds.set_target_bounds(min_bound, max_bound);
                        } else {
                            self.highlight_bounds = Some(HighlightBounds::new(min_bound, max_bound));
                        }
                    }
                },
                InstrumentCommand::SetBothNeedles(primary, secondary) => {
                    self.current_values.insert("needle1".to_string(), primary);
                    self.current_values.insert("needle2".to_string(), secondary);
                    
                    // Update primary needle
                    if self.needle1.is_none() {
                        self.needle1 = Some(Needle::new());
                    }
                    if let Some(ref mut needle) = self.needle1 {
                        let target_pos = ((primary - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                        needle.set_target_pos(target_pos);
                    }
                    
                    // Update secondary needle
                    if self.needle2.is_none() {
                        self.needle2 = Some(Needle::new());
                    }
                    if let Some(ref mut needle) = self.needle2 {
                        let target_pos = ((secondary - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0);
                        needle.set_target_pos(target_pos);
                    }
                },
            }
        }

        // Update needle positions
        if let Some(ref mut needle) = self.needle1 {
            needle.update_position();
        }
        if let Some(ref mut needle) = self.needle2 {
            needle.update_position();
        }
        
        // Update highlight bounds position
        if let Some(ref mut bounds) = self.highlight_bounds {
            bounds.update_position();
        }
    }

    fn is_out_of_range(&self) -> bool {
        // Check if any needle values are out of range
        if let Some(value) = self.current_values.get("needle1") {
            if *value < self.min_value || *value > self.max_value {
                return true;
            }
        }
        if let Some(value) = self.current_values.get("needle2") {
            if *value < self.min_value || *value > self.max_value {
                return true;
            }
        }
        false
    }

    fn set_highlight_override(&mut self, lower: f64, upper: f64) {
        let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
        self.highlight_override = Some((min_bound, max_bound));
        // Set the initial highlight bounds to the override values
        if let Some(ref mut bounds) = self.highlight_bounds {
            bounds.set_target_bounds(min_bound, max_bound);
        } else {
            self.highlight_bounds = Some(HighlightBounds::new(min_bound, max_bound));
        }
    }
}

struct Dial {
    cx: i32,
    cy: i32,
    r: i32,
    thickness: i32,
    arc_span: f64,
    start_angle: f64,
}

impl Dial {
    fn new(width: usize, height: usize, config: &InstrumentConfig) -> Self {
        let cx = width as i32 / 2;
        let cy = height as i32 / 2;
        let r = (width.min(height) as i32) / 2 - config.dial_margin;
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self { cx, cy, r, thickness: config.dial_thickness, arc_span, start_angle }
    }

    fn new_complication(width: usize, height: usize, config: &InstrumentConfig) -> Self {
        // Create a smaller dial for the needle2 complication
        let r = ((width.min(height) as f64) / config.secondary_dial_size) as i32 - config.secondary_dial_margin; // Much smaller radius
        let cx = width as i32 / 2; // Center horizontally
        let cy = r + config.secondary_dial_margin + config.secondary_dial_shift; // Position in top middle with configurable offset
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self { cx, cy, r, thickness: config.secondary_dial_thickness, arc_span, start_angle }
    }
}

struct Needle {
    pos: f64, // Normalized [0,1]
    target_pos: f64,
    phase: f64,
}

impl Needle {
    fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            pos: 0.5,
            target_pos: 0.5,
            phase: rng.random_range(0.0..1000.0),
        }
    }

    fn set_target_pos(&mut self, target: f64) {
        self.target_pos = target.clamp(0.0, 1.0);
    }

    fn update_random(&mut self) {
        let mut rng = rand::rng();
        self.phase += rng.random_range(0.0..1000.0);
        if rng.random_range(0.0..1.0) < 0.01 {
            self.target_pos = rng.random_range(0.0..1.0);
        }
    }

    fn update_position(&mut self) {
        self.pos = lerp(self.pos, self.target_pos).clamp(0.0, 1.0);
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// ============================================================================
// RENDERING AND DRAWING FUNCTIONS
// ============================================================================

fn render_instrument(canvas: &mut Canvas, state: &AppState, config: &InstrumentConfig) {
    // Create a scene to build our drawing commands
    let mut scene = Scene::new(canvas.width, canvas.height);
    
    // Add clear command
    scene.add_command(DrawCommand::Clear((0xff, 0xff, 0xff)));
    
    let dial = Dial::new(canvas.width, canvas.height, config);
    let is_out_of_range = state.is_out_of_range();
    let base_color = if is_out_of_range { (0xff, 0x00, 0x00) } else { (0x00, 0x00, 0x00) };
    
    // Build dial drawing commands
    build_dial_commands(&mut scene, &dial, (state.min_value, state.max_value), base_color, state.highlight_bounds.as_ref().map(|b| b.get_bounds()), config);
    
    // Add curved text at the top of the dial
    build_curved_text_commands(&mut scene, &dial, base_color, config);
    
    // Draw needle1 if it exists
    if let Some(ref needle) = state.needle1 {
        let needle_color = if is_out_of_range { 
            (0xff, 0x00, 0x00) 
        } else { 
            (0x00, 0x00, 0x00) // Black for needle1
        };
        build_needle_commands(&mut scene, needle, &dial, needle_color, config);
    }
    
    // Draw needle2 if it exists
    if let Some(ref needle) = state.needle2 {
        let needle_color = if is_out_of_range { 
            (0xff, 0x00, 0x00) 
        } else { 
            (0x00, 0x7f, 0xff) // Blue for needle2
        };
        
        if config.use_secondary_complication {
            // Draw needle2 on a smaller complication dial in the top middle
            let complication_dial = Dial::new_complication(canvas.width, canvas.height, config);
            build_complication_dial_commands(&mut scene, &complication_dial, (state.min_value, state.max_value), (0x00, 0x00, 0x00), config); // Black dial
            build_complication_needle_commands(&mut scene, needle, &complication_dial, (0x00, 0x00, 0x00), config);
        } else {
            // Draw needle2 on the main dial (original behavior)
            build_needle_commands(&mut scene, needle, &dial, needle_color, config);
        }
    }
    
    // Draw readout if readout value exists
    if let Some(&value) = state.current_values.get("readout") {
        build_readout_commands(&mut scene, canvas, value, base_color, config);
    }
    
    // Draw warning indicator
    if is_out_of_range {
        build_warning_commands(&mut scene, &dial, config);
    }
    
    // Render the complete scene
    scene.render(canvas, config);
}

// ============================================================================
// RETAINED MODE BUILDER FUNCTIONS
// ============================================================================

fn build_dial_commands(scene: &mut Scene, dial: &Dial, range: (f64, f64), color: (u8, u8, u8), highlight_range: Option<(f64, f64)>, config: &InstrumentConfig) {
    // Add highlight band if needed
    if let Some(highlight) = highlight_range {
        let (hl_start, hl_end) = highlight;
        let (r_start, r_end) = range;

        // Convert to normalized [0,1] range
        let norm_hl_start = ((hl_start - r_start) / (r_end - r_start)).clamp(0.0, 1.0);
        let norm_hl_end = ((hl_end - r_start) / (r_end - r_start)).clamp(0.0, 1.0);

        // Calculate angles for the highlight band
        let start_angle = dial.start_angle + dial.arc_span * norm_hl_start;
        let end_angle = dial.start_angle + dial.arc_span * norm_hl_end;

        scene.add_command(DrawCommand::HighlightBand {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            start_angle,
            end_angle,
            inner_radius: config.highlight_band_width as f64,
            outer_radius: 0.0,
        });
    }
    
    // Add main dial arc
    scene.add_command(DrawCommand::Arc {
        cx: dial.cx,
        cy: dial.cy,
        r: dial.r,
        thickness: dial.thickness,
        start_angle: dial.start_angle,
        arc_span: dial.arc_span,
        color,
    });
    
    // Add ticks and labels
    for i in 0..config.ticks_count {
        let t = i as f64 / (config.ticks_count as f64 - 1.0);
        let angle = dial.start_angle + dial.arc_span * t;
        
        // Major tick
        scene.add_command(DrawCommand::Tick {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            angle,
            length: config.major_tick_length,
            thickness: config.major_tick_thickness,
            color,
        });
        
        // Minor ticks
        if i < config.ticks_count - 1 {
            for j in 1..=config.minor_ticks_per_interval {
                let minor_t = t + (j as f64 / (config.minor_ticks_per_interval as f64 * (config.ticks_count as f64 - 1.0)));
                let minor_angle = dial.start_angle + dial.arc_span * minor_t;
                scene.add_command(DrawCommand::Tick {
                    cx: dial.cx,
                    cy: dial.cy,
                    r: dial.r,
                    angle: minor_angle,
                    length: config.minor_tick_length,
                    thickness: config.minor_tick_thickness,
                    color,
                });
            }
        }
        
        // Number labels
        let label_radius = dial.r as f64 - config.major_tick_length as f64 - config.dial_ticks_to_numbers_distance;
        let label_x = dial.cx as f64 + angle.cos() * label_radius;
        let label_y = dial.cy as f64 + angle.sin() * label_radius;
        let value = range.0 + t * (range.1 - range.0);
        let value_str = format!("{}", value.round() as i64);
        scene.add_command(DrawCommand::Text {
            x: label_x as i32,
            y: label_y as i32,
            text: value_str,
            font_size: config.dial_numbers_font_size,
            color,
        });
    }
}

fn build_needle_commands(scene: &mut Scene, needle: &Needle, dial: &Dial, color: (u8, u8, u8), config: &InstrumentConfig) {
    let angle = dial.start_angle + dial.arc_span * needle.pos;
    let needle_length = dial.r as f64 * config.needle_length_factor;
    let nx = (dial.cx as f64 + angle.cos() * needle_length) as i32;
    let ny = (dial.cy as f64 + angle.sin() * needle_length) as i32;

    // Main needle line
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: nx,
        y1: ny,
        thickness: config.needle_width,
        tapered: true,
        color,
    });

    // Draw the needle's back extension
    let back_length = config.needle_back_length;
    let back_x = (dial.cx as f64 - angle.cos() * back_length) as i32;
    let back_y = (dial.cy as f64 - angle.sin() * back_length) as i32;
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: back_x,
        y1: back_y,
        thickness: config.needle_width,
        tapered: false,
        color,
    });

    // Draw the needle's dot
    let dot_radius = config.dot_radius as i32;
    scene.add_command(DrawCommand::Circle {
        cx: dial.cx,
        cy: dial.cy,
        radius: dot_radius,
        color,
    });
}

fn build_complication_dial_commands(scene: &mut Scene, dial: &Dial, range: (f64, f64), color: (u8, u8, u8), config: &InstrumentConfig) {
    // Add main dial arc using mini_dial constants
    scene.add_command(DrawCommand::Arc {
        cx: dial.cx,
        cy: dial.cy,
        r: dial.r,
        thickness: dial.thickness,
        start_angle: dial.start_angle,
        arc_span: dial.arc_span,
        color,
    });
    
    // Add ticks and labels using mini_dial constants
    for i in 0..config.secondary_ticks_count {
        let t = i as f64 / (config.secondary_ticks_count as f64 - 1.0);
        let angle = dial.start_angle + dial.arc_span * t;
        
        // Major tick
        scene.add_command(DrawCommand::Tick {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            angle,
            length: config.secondary_tick_length,
            thickness: config.secondary_major_tick_thickness,
            color,
        });
        
        // Minor ticks
        if i < config.secondary_ticks_count - 1 {
            for j in 1..=config.secondary_minor_ticks_per_interval {
                let minor_t = t + (j as f64 / (config.secondary_minor_ticks_per_interval as f64 * (config.secondary_ticks_count as f64 - 1.0)));
                let minor_angle = dial.start_angle + dial.arc_span * minor_t;
                scene.add_command(DrawCommand::Tick {
                    cx: dial.cx,
                    cy: dial.cy,
                    r: dial.r,
                    angle: minor_angle,
                    length: config.secondary_minor_tick_length,
                    thickness: config.secondary_minor_tick_thickness,
                    color,
                });
            }
        }
        
        // Number labels with smaller font
        let label_radius = dial.r as f64 - config.secondary_tick_length as f64 - config.secondary_dial_ticks_to_numbers_distance;
        let label_x = dial.cx as f64 + angle.cos() * label_radius;
        let label_y = dial.cy as f64 + angle.sin() * label_radius;
        let value = range.0 + t * (range.1 - range.0);
        let value_str = format!("{}", value.round() as i64);
        scene.add_command(DrawCommand::Text {
            x: label_x as i32,
            y: label_y as i32,
            text: value_str,
            font_size: config.secondary_dial_numbers_font_size,
            color,
        });
    }
}

fn build_complication_needle_commands(scene: &mut Scene, needle: &Needle, dial: &Dial, color: (u8, u8, u8), config: &InstrumentConfig) {
    let angle = dial.start_angle + dial.arc_span * needle.pos;
    let needle_length = dial.r as f64 * config.secondary_needle_length_factor;
    let nx = (dial.cx as f64 + angle.cos() * needle_length) as i32;
    let ny = (dial.cy as f64 + angle.sin() * needle_length) as i32;

    // Main needle line using mini_dial constants
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: nx,
        y1: ny,
        thickness: config.secondary_needle_width,
        tapered: true,
        color,
    });

    // Draw the needle's back extension using mini_dial constants
    let back_length = config.secondary_needle_back_length;
    let back_x = (dial.cx as f64 - angle.cos() * back_length) as i32;
    let back_y = (dial.cy as f64 - angle.sin() * back_length) as i32;
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: back_x,
        y1: back_y,
        thickness: config.secondary_needle_width,
        tapered: false,
        color,
    });

    // Draw the needle's dot using mini_dial constants
    let dot_radius = config.secondary_dial_dot_radius as i32;
    scene.add_command(DrawCommand::Circle {
        cx: dial.cx,
        cy: dial.cy,
        radius: dot_radius,
        color,
    });
}

fn build_readout_commands(scene: &mut Scene, canvas: &Canvas, value: f64, color: (u8, u8, u8), config: &InstrumentConfig) {
    let value_int = value.trunc() as i32;
    let value_frac = ((value.fract() * 1000.0).round() as u32).min(999);
    let label_x = (canvas.width as f64 * config.readout_x_factor) as i32;
    let label_y = (canvas.height as f64 * config.readout_y_factor) as i32;

    // Integer part in big font
    let value_str = format!("{}", value_int);
    scene.add_command(DrawCommand::Text {
        x: label_x,
        y: label_y,
        text: value_str.clone(),
        font_size: config.readout_big_font_size,
        color,
    });

    // Fractional part in smaller font
    let font = Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
    let scale_big = Scale::uniform(config.readout_big_font_size);
    let int_width = calculate_text_width(&value_str, &font, scale_big);
    let frac_str = format!("{:03}", value_frac);
    let frac_x = label_x + int_width / 2 + 28;
    let frac_y = label_y + 2;
    scene.add_command(DrawCommand::Text {
        x: frac_x,
        y: frac_y,
        text: frac_str,
        font_size: config.readout_small_font_size,
        color,
    });

    // TODO: Add readout box drawing commands
    // For now, we'll keep the immediate mode box drawing in a separate helper
    build_readout_box_commands(scene, label_x, label_y, frac_x, frac_y, &value_str, color, config);
}

fn build_readout_box_commands(scene: &mut Scene, label_x: i32, label_y: i32, frac_x: i32, frac_y: i32, int_str: &str, color: (u8, u8, u8), config: &InstrumentConfig) {
    let box_padding = config.readout_box_padding;
    let box_thickness = config.readout_box_thickness;
    let font_size = (config.readout_big_font_size / 11.0) as i32;
    
    let box_left = label_x - box_padding - font_size * int_str.len() as i32;
    let box_top = label_y - box_padding;
    let box_right = frac_x + box_padding + 5;
    let box_bottom = frac_y + box_padding;

    // Draw box lines as needle lines
    scene.add_command(DrawCommand::NeedleLine {
        x0: box_left, y0: box_top, x1: box_right, y1: box_top,
        thickness: box_thickness as f32, tapered: false, color
    });
    scene.add_command(DrawCommand::NeedleLine {
        x0: box_left, y0: box_bottom, x1: box_right, y1: box_bottom,
        thickness: box_thickness as f32, tapered: false, color
    });
    scene.add_command(DrawCommand::NeedleLine {
        x0: box_left, y0: box_top, x1: box_left, y1: box_bottom,
        thickness: box_thickness as f32, tapered: false, color
    });
    scene.add_command(DrawCommand::NeedleLine {
        x0: box_right, y0: box_top, x1: box_right, y1: box_bottom,
        thickness: box_thickness as f32, tapered: false, color
    });
}

fn build_warning_commands(scene: &mut Scene, dial: &Dial, config: &InstrumentConfig) {
    let exclamation_x = dial.cx;
    let exclamation_y = dial.cy - (dial.r / 4);
    let exclamation_str = "!";
    let exclamation_color = (0xff, 0x00, 0x00);
    scene.add_command(DrawCommand::Text {
        x: exclamation_x,
        y: exclamation_y,
        text: exclamation_str.to_string(),
        font_size: config.exclamation_mark_size,
        color: exclamation_color,
    });
}

fn build_curved_text_commands(scene: &mut Scene, dial: &Dial, color: (u8, u8, u8), config: &InstrumentConfig) {
    let text_radius = dial.r as f64 + config.curved_text_radius_offset;
    let text_start_angle = config.curved_text_angle; // Use the configured center angle directly
    
    scene.add_command(DrawCommand::CurvedText {
        cx: dial.cx,
        cy: dial.cy,
        radius: text_radius,
        text: config.curved_text.to_string(),
        font_size: config.curved_text_font_size,
        arc_span: config.curved_text_arc_span,
        start_angle: text_start_angle,
        color,
    });
}

fn calculate_text_width(text: &str, font: &Font, scale: Scale) -> i32 {
    use rusttype::{point, PositionedGlyph};
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let (min_x, max_x, _, _) = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y), bb| {
            (min_x.min(bb.min.x), max_x.max(bb.max.x), min_y.min(bb.min.y), max_y.max(bb.max.y))
        },
    );
    if min_x < max_x { max_x - min_x } else { 0 }
}

fn lerp(current: f64, target: f64) -> f64 {
    current + (target - current) * 0.1 // Default lerp factor for general animations
}

// ============================================================================
// DRAWING PRIMITIVES
// ============================================================================

fn set_pixel(frame: &mut [u8], width: usize, x: usize, y: usize, r: u8, g: u8, b: u8, alpha: f32) {
    if x < width && y < frame.len() / (width * 4) {
        let idx = (y * width + x) * 4;
        let src = [r as f32, g as f32, b as f32, 255.0 * alpha];
        let dst = [frame[idx] as f32, frame[idx + 1] as f32, frame[idx + 2] as f32, frame[idx + 3] as f32];
        let a = src[3] / 255.0;
        let out = [
            (src[0] * a + dst[0] * (1.0 - a)).round() as u8,
            (src[1] * a + dst[1] * (1.0 - a)).round() as u8,
            (src[2] * a + dst[2] * (1.0 - a)).round() as u8,
            0xff,
        ];
        frame[idx..idx + 4].copy_from_slice(&out);
    }
}

fn draw_thick_line_aa(frame: &mut [u8], width: usize, x0: i32, y0: i32, x1: i32, y1: i32, thickness: f32, r: u8, g: u8, b: u8) {
    let min_x = x0.min(x1) - thickness.ceil() as i32 - 1;
    let max_x = x0.max(x1) + thickness.ceil() as i32 + 1;
    let min_y = y0.min(y1) - thickness.ceil() as i32 - 1;
    let max_y = y0.max(y1) + thickness.ceil() as i32 + 1;
    let dx = (x1 - x0) as f32;
    let dy = (y1 - y0) as f32;
    let len_sq = dx * dx + dy * dy;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 - x0 as f32;
            let py = y as f32 - y0 as f32;
            let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
            let lx = x0 as f32 + t * dx;
            let ly = y0 as f32 + t * dy;
            let dist = ((lx - x as f32).powi(2) + (ly - y as f32).powi(2)).sqrt();
            let aa = (1.0 - (dist - thickness / 2.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            if aa > 0.01 {
                set_pixel(frame, width, x as usize, y as usize, r, g, b, aa);
            }
        }
    }
}

fn draw_thick_line_tapered_aa(frame: &mut [u8], width: usize, x0: i32, y0: i32, x1: i32, y1: i32, thickness: f32, r: u8, g: u8, b: u8) {
    let min_x = x0.min(x1) - thickness.ceil() as i32 - 1;
    let max_x = x0.max(x1) + thickness.ceil() as i32 + 1;
    let min_y = y0.min(y1) - thickness.ceil() as i32 - 1;
    let max_y = y0.max(y1) + thickness.ceil() as i32 + 1;
    let dx = (x1 - x0) as f32;
    let dy = (y1 - y0) as f32;
    let len_sq = dx * dx + dy * dy;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 - x0 as f32;
            let py = y as f32 - y0 as f32;
            let t = ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0);
            let lx = x0 as f32 + t * dx;
            let ly = y0 as f32 + t * dy;
            let dist = ((lx - x as f32).powi(2) + (ly - y as f32).powi(2)).sqrt();
            let local_thickness = thickness * (1.0 - t * 0.95); // 0.05 to avoid vanishing too soon
            let aa = (1.0 - (dist - local_thickness / 2.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            if aa > 0.01 {
                set_pixel(frame, width, x as usize, y as usize, r, g, b, aa);
            }
        }
    }
}

fn draw_text(frame: &mut [u8], width: usize, height: usize, x: i32, y: i32, text: &str, font: &rusttype::Font, scale: rusttype::Scale, color: (u8, u8, u8)) {
    use rusttype::{point, PositionedGlyph};
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, point(0.0, 0.0 + v_metrics.ascent)).collect();
    // Calculate bounding box for the whole string

    let (min_x, max_x, min_y, max_y) = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y), bb| {
            (
                min_x.min(bb.min.x),
                max_x.max(bb.max.x),
                min_y.min(bb.min.y),
                max_y.max(bb.max.y),
            )
        },
    );
    let width_px = if min_x < max_x { max_x - min_x } else { 0 };
    let height_px = if min_y < max_y { max_y - min_y } else { 0 };
    let offset_x = x - width_px / 2;
    let offset_y = y - height_px / 2;
    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = offset_x + gx as i32 + bb.min.x - min_x;
                let py = offset_y + gy as i32 + bb.min.y - min_y;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    set_pixel(frame, width, px as usize, py as usize, color.0, color.1, color.2, v as f32);
                }
            });
        }
    }
}

fn draw_curved_text(canvas: &mut Canvas, cx: i32, cy: i32, radius: f64, text: &str, font: &rusttype::Font, scale: rusttype::Scale, arc_span: f64, center_angle: f64, color: (u8, u8, u8)) {
    use rusttype::{point, PositionedGlyph};
    
    // Create glyphs for the text to calculate individual character positions
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, point(0.0, 0.0 + v_metrics.ascent)).collect();
    
    if glyphs.is_empty() {
        return;
    }
    
    // Calculate total text width by examining glyph positions
    let total_width = if let (Some(first), Some(last)) = (glyphs.first(), glyphs.last()) {
        (last.position().x - first.position().x + last.unpositioned().h_metrics().advance_width) as f64
    } else {
        0.0
    };
    
    if total_width <= 0.0 {
        return;
    }
    
    // Calculate the actual arc span needed for the text
    let chars_arc_span = total_width / radius;
    let actual_arc_span = chars_arc_span.min(arc_span);
    
    // Start angle for the text (center the text around center_angle)
    let start_angle = center_angle - actual_arc_span / 2.0;
    
    // Draw each character
    for glyph in &glyphs {
        if glyph.pixel_bounding_box().is_some() {
            let char_advance = glyph.unpositioned().h_metrics().advance_width as f64;
            
            // Calculate the angle for the center of this character
            let char_position = glyph.position().x as f64;
            let first_position = glyphs[0].position().x as f64;
            let relative_position = char_position - first_position + char_advance / 2.0;
            let char_angle = start_angle + (relative_position / radius);
            
            // Position on the arc
            let char_x = cx as f64 + char_angle.cos() * radius;
            let char_y = cy as f64 + char_angle.sin() * radius;
            
            // Rotation angle (tangent to the circle)
            let rotation_angle = char_angle + std::f64::consts::FRAC_PI_2;
            
            // Draw the character with improved rotation
            draw_rotated_glyph_improved(canvas, glyph, char_x, char_y, rotation_angle, color);
        }
    }
}

fn draw_rotated_glyph_improved(canvas: &mut Canvas, glyph: &rusttype::PositionedGlyph, center_x: f64, center_y: f64, rotation: f64, color: (u8, u8, u8)) {
    if let Some(bb) = glyph.pixel_bounding_box() {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        
        // Calculate glyph center offset
        let glyph_center_x = (bb.min.x + bb.max.x) as f64 / 2.0;
        let glyph_center_y = (bb.min.y + bb.max.y) as f64 / 2.0;
        
        // Draw each pixel of the glyph with sub-pixel accuracy
        glyph.draw(|gx, gy, v| {
            if v > 0.001 {  // Lower threshold for better coverage
                // Get pixel position relative to glyph origin
                let px = gx as f64 + bb.min.x as f64;
                let py = gy as f64 + bb.min.y as f64;
                
                // Translate to glyph center
                let local_x = px - glyph_center_x;
                let local_y = py - glyph_center_y;
                
                // Apply rotation
                let rotated_x = local_x * cos_r - local_y * sin_r;
                let rotated_y = local_x * sin_r + local_y * cos_r;
                
                // Translate to final position
                let final_x = center_x + rotated_x;
                let final_y = center_y + rotated_y;
                
                // Draw with sub-pixel positioning for smoother rendering
                draw_antialiased_pixel(canvas, final_x, final_y, color, v as f32);
            }
        });
    }
}

fn draw_antialiased_pixel(canvas: &mut Canvas, x: f64, y: f64, color: (u8, u8, u8), alpha: f32) {
    // Get the integer coordinates
    let x_floor = x.floor() as i32;
    let y_floor = y.floor() as i32;
    
    // Calculate fractional parts for sub-pixel positioning
    let x_frac = x - x_floor as f64;
    let y_frac = y - y_floor as f64;
    
    // Distribute the pixel across the 4 nearest pixels with bilinear interpolation
    let samples = [
        (x_floor, y_floor, (1.0 - x_frac) * (1.0 - y_frac)),
        (x_floor + 1, y_floor, x_frac * (1.0 - y_frac)),
        (x_floor, y_floor + 1, (1.0 - x_frac) * y_frac),
        (x_floor + 1, y_floor + 1, x_frac * y_frac),
    ];
    
    for (px, py, weight) in samples.iter() {
        if *px >= 0 && *px < canvas.width as i32 && *py >= 0 && *py < canvas.height as i32 {
            let final_alpha = alpha * (*weight as f32);
            if final_alpha > 0.001 {  // Lower threshold for better coverage
                set_pixel(canvas.frame, canvas.width, *px as usize, *py as usize, color.0, color.1, color.2, final_alpha);
            }
        }
    }
}

fn draw_circle(frame: &mut [u8], width: usize, cx: i32, cy: i32, radius: i32, r: u8, g: u8, b: u8) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            let dist = ((x * x + y * y) as f64).sqrt();
            let aa = if dist > radius as f64 {
                1.0 - (dist - radius as f64).min(1.0)
            } else {
                1.0
            };
            if dist <= radius as f64 + 1.0 && aa > 0.0 {
                let px = cx + x;
                let py = cy + y;
                if px >= 0 && py >= 0 && (px as usize) < width && (py as usize) < frame.len() / (width * 4) {
                    set_pixel(frame, width, px as usize, py as usize, r, g, b, aa as f32);
                }
            }
        }
    }
}

fn render_arc_immediate(canvas: &mut Canvas, cx: i32, cy: i32, r: i32, thickness: i32, start_angle: f64, arc_span: f64, color: (u8, u8, u8)) {
    let end_angle = start_angle + arc_span;
    let mut start_angle = start_angle;
    let mut end_angle = end_angle;
    if start_angle < 0.0 { start_angle += 2.0 * std::f64::consts::PI; }
    if end_angle >= 2.0 * std::f64::consts::PI { end_angle -= 2.0 * std::f64::consts::PI; }
    
    for y in 0..canvas.height as i32 {
        for x in 0..canvas.width as i32 {
            let dx = x - cx;
            let dy = y - cy;
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let mut angle = (dy as f64).atan2(dx as f64);
            if angle < 0.0 {
                angle += 2.0 * std::f64::consts::PI;
            }
            let mut start = start_angle;
            let mut end = end_angle;
            if start < 0.0 { start += 2.0 * std::f64::consts::PI; }
            if end < 0.0 { end += 2.0 * std::f64::consts::PI; }
            let in_arc = if start < end {
                angle >= start && angle <= end
            } else {
                angle >= start || angle <= end
            };
            if in_arc {
                let aa = if dist > r as f64 {
                    1.0 - (dist - r as f64).min(1.0)
                } else if dist < (r - thickness) as f64 {
                    1.0 - ((r - thickness) as f64 - dist).min(1.0)
                } else {
                    1.0
                };
                if dist >= (r - thickness - 1) as f64 && dist <= (r + 1) as f64 && aa > 0.0 {
                    set_pixel(canvas.frame, canvas.width, x as usize, y as usize, color.0, color.1, color.2, aa as f32);
                }
            }
        }
    }
}

fn render_highlight_band_immediate(canvas: &mut Canvas, cx: i32, cy: i32, r: i32, start_angle: f64, end_angle: f64, inner_radius: f64, outer_radius: f64, config: &InstrumentConfig) {
    // Draw the highlight band as a thick arc
    let band_inner_radius = (r as f64 - inner_radius).max(0.0);
    let band_outer_radius = (r as f64 - outer_radius).max(0.0);
    
    for y in 0..canvas.height as i32 {
        for x in 0..canvas.width as i32 {
            let dx = x - cx;
            let dy = y - cy;
            let dist = ((dx * dx + dy * dy) as f64).sqrt();
            let mut angle = (dy as f64).atan2(dx as f64);
            if angle < 0.0 {
                angle += 2.0 * std::f64::consts::PI;
            }
            
            // Calculate angular distance to edges for anti-aliasing
            let mut angular_alpha = 1.0;
            if start_angle <= end_angle {
                // Normal case: start < end
                if angle < start_angle {
                    angular_alpha = 1.0 - ((start_angle - angle).min(config.highlight_band_edge_softness) / config.highlight_band_edge_softness);
                } else if angle > end_angle {
                    angular_alpha = 1.0 - ((angle - end_angle).min(config.highlight_band_edge_softness) / config.highlight_band_edge_softness);
                }
                if angle < start_angle || angle > end_angle {
                    angular_alpha = angular_alpha.max(0.0);
                }
            } else {
                // Wrap case: start > end (crosses 0 degrees)
                if angle < end_angle {
                    // Close to end edge
                    angular_alpha = 1.0 - ((end_angle - angle).min(config.highlight_band_edge_softness) / config.highlight_band_edge_softness).max(0.0);
                } else if angle > start_angle {
                    // Close to start edge  
                    angular_alpha = 1.0 - ((angle - start_angle).min(config.highlight_band_edge_softness) / config.highlight_band_edge_softness).max(0.0);
                } else {
                    // Between end and start (outside the arc)
                    let dist_to_start = if start_angle > angle {
                        start_angle - angle
                    } else {
                        2.0 * std::f64::consts::PI - angle + start_angle
                    };
                    let dist_to_end = if angle > end_angle {
                        angle - end_angle
                    } else {
                        end_angle + 2.0 * std::f64::consts::PI - angle
                    };
                    let min_dist = dist_to_start.min(dist_to_end);
                    angular_alpha = 1.0 - (min_dist.min(config.highlight_band_edge_softness) / config.highlight_band_edge_softness);
                    angular_alpha = angular_alpha.max(0.0);
                }
            }
            
            // Calculate radial alpha with improved anti-aliasing
            let radial_alpha = if dist < band_inner_radius - 1.0 {
                0.0
            } else if dist < band_inner_radius + 1.0 {
                // Smooth transition at inner edge
                ((dist - (band_inner_radius - 1.0)) / 2.0).clamp(0.0, 1.0)
            } else if dist <= band_outer_radius - 1.0 {
                1.0
            } else if dist <= band_outer_radius + 1.0 {
                // Smooth transition at outer edge
                1.0 - ((dist - (band_outer_radius - 1.0)) / 2.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            
            let final_alpha = (angular_alpha * radial_alpha * config.highlight_band_alpha).clamp(0.0, 1.0);
            
            if final_alpha > 0.01 {
                let color = config.highlight_band_color.as_tuple();
                set_pixel(canvas.frame, canvas.width, x as usize, y as usize, 
                        color.0, color.1, color.2, final_alpha as f32);
            }
        }
    }
}

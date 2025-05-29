// ============================================================================
// CRATE CONFIGURATION & IMPORTS
// ============================================================================

// External crate imports
use bon::Builder;
use pixels::{Pixels, SurfaceTexture};
use rusttype::{Font, Scale};

// Standard library imports
use std::sync::mpsc::Receiver;
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
    SetChronograph(f64),
    SetSecondaryChronograph(f64),
    SetReadout(f64),
    SetHighlightBounds(f64, f64),
    SetBothNeedles(f64, f64),          // primary, secondary
    SetAllNeedles(f64, f64, f64, f64), // primary, secondary, chronograph, secondary_chronograph
    SetBothChronographs(f64, f64),     // chronograph, secondary_chronograph
}

/// Main instrument struct - the primary public interface
#[derive(Debug, Clone)]
pub struct Instrument {
    config: InstrumentConfig,
    state: InstrumentState,
}

#[derive(Debug, Clone, Builder)]
pub struct InstrumentConfig {
    #[builder(default = "".to_string())]
    pub title: String,
    #[builder(default = (0.0, 100.0))]
    pub range: (f64, f64),
    pub highlight_band: Option<(f64, f64, Color)>,

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

    // Chronograph configuration
    #[builder(default = (0.0, 60.0))]
    pub chronograph_range: (f64, f64),
    #[builder(default = 130)]
    pub chronograph_dial_shift: i32,
    #[builder(default = 7.0)]
    pub chronograph_dial_size: f64,
    #[builder(default = 5)]
    pub chronograph_ticks_count: usize,
    #[builder(default = 10)]
    pub chronograph_tick_length: i32,
    #[builder(default = 15)]
    pub chronograph_dial_margin: i32,
    #[builder(default = 2)]
    pub chronograph_dial_thickness: i32,
    #[builder(default = 1.0)]
    pub chronograph_needle_length_factor: f64,
    #[builder(default = 4.0)]
    pub chronograph_needle_width: f32,
    #[builder(default = 30.0)]
    pub chronograph_needle_back_length: f64,
    #[builder(default = 30.0)]
    pub chronograph_dial_numbers_font_size: f32,
    #[builder(default = 30.0)]
    pub chronograph_dial_ticks_to_numbers_distance: f64,
    #[builder(default = 8)]
    pub chronograph_dial_dot_radius: i32,
    #[builder(default = 0)]
    pub chronograph_minor_ticks_per_interval: usize,
    #[builder(default = 4)]
    pub chronograph_minor_tick_length: i32,
    #[builder(default = 2.0)]
    pub chronograph_major_tick_thickness: f32,
    #[builder(default = 0.5)]
    pub chronograph_minor_tick_thickness: f32,

    // Secondary Chronograph configuration
    #[builder(default = (0.0, 60.0))]
    pub secondary_chronograph_range: (f64, f64),
    #[builder(default = 90)]
    pub secondary_chronograph_dial_shift: i32,
    #[builder(default = 7.0)]
    pub secondary_chronograph_dial_size: f64,
    #[builder(default = 5)]
    pub secondary_chronograph_ticks_count: usize,
    #[builder(default = 10)]
    pub secondary_chronograph_tick_length: i32,
    #[builder(default = 15)]
    pub secondary_chronograph_dial_margin: i32,
    #[builder(default = 2)]
    pub secondary_chronograph_dial_thickness: i32,
    #[builder(default = 1.0)]
    pub secondary_chronograph_needle_length_factor: f64,
    #[builder(default = 4.0)]
    pub secondary_chronograph_needle_width: f32,
    #[builder(default = 30.0)]
    pub secondary_chronograph_needle_back_length: f64,
    #[builder(default = 30.0)]
    pub secondary_chronograph_dial_numbers_font_size: f32,
    #[builder(default = 30.0)]
    pub secondary_chronograph_dial_ticks_to_numbers_distance: f64,
    #[builder(default = 8)]
    pub secondary_chronograph_dial_dot_radius: i32,
    #[builder(default = 0)]
    pub secondary_chronograph_minor_ticks_per_interval: usize,
    #[builder(default = 4)]
    pub secondary_chronograph_minor_tick_length: i32,
    #[builder(default = 2.0)]
    pub secondary_chronograph_major_tick_thickness: f32,
    #[builder(default = 0.5)]
    pub secondary_chronograph_minor_tick_thickness: f32,

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
    #[builder(default = "".to_string())]
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
    #[builder(default = 20)]
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
    chronograph_value: Option<f64>,
    secondary_chronograph_value: Option<f64>,
    readout_value: Option<f64>,
}

impl Instrument {
    pub fn set_value(&mut self, value: f64) {
        self.state.primary_value = value.clamp(self.config.range.0, self.config.range.1);
        self.state.readout_value = Some(self.state.primary_value);
    }

    pub fn set_primary_value(&mut self, value: f64) {
        self.state.primary_value = value.clamp(self.config.range.0, self.config.range.1);
    }

    pub fn set_secondary_value(&mut self, value: f64) {
        let clamped_value = value.clamp(self.config.range.0, self.config.range.1);
        self.state.secondary_value = Some(clamped_value);
    }

    pub fn set_chronograph_value(&mut self, value: f64) {
        let clamped_value = value.clamp(
            self.config.chronograph_range.0,
            self.config.chronograph_range.1,
        );
        self.state.chronograph_value = Some(clamped_value);
    }

    pub fn set_secondary_chronograph_value(&mut self, value: f64) {
        let clamped_value = value.clamp(
            self.config.secondary_chronograph_range.0,
            self.config.secondary_chronograph_range.1,
        );
        self.state.secondary_chronograph_value = Some(clamped_value);
    }

    pub fn show(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let title = self.config.title.clone();
        let range = self.config.range;
        let highlight_range = self
            .config
            .highlight_band
            .map(|(min, max, _color)| (min, max));

        self.run_window(title, range, highlight_range, None)
    }

    pub fn show_with_commands(
        &mut self,
        receiver: Receiver<InstrumentCommand>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let title = self.config.title.clone();
        let range = self.config.range;
        let highlight_range = self
            .config
            .highlight_band
            .map(|(min, max, _color)| (min, max));

        self.run_window(title, range, highlight_range, Some(receiver))
    }

    fn run_window(
        &self,
        title: String,
        range: (f64, f64),
        highlight_range: Option<(f64, f64)>,
        receiver: Option<Receiver<InstrumentCommand>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let logical_width: usize = self.config.window_width;
        let logical_height: usize = self.config.window_height;

        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(LogicalSize::new(
                logical_width as f64,
                logical_height as f64,
            ))
            .with_resizable(false)
            .build(&event_loop)?;

        let window = std::sync::Arc::new(window);

        let mut app_state = AppState::new(range.0, range.1);
        if let Some((lower, upper)) = highlight_range {
            app_state.set_highlight_override(lower, upper);
        }

        // Initialize app_state with current instrument state
        app_state.set_primary_value(self.state.primary_value);
        if let Some(secondary) = self.state.secondary_value {
            app_state.set_secondary_value(secondary);
        }
        if let Some(chronograph) = self.state.chronograph_value {
            app_state.set_chronograph_value(chronograph);
        }
        if let Some(secondary_chronograph) = self.state.secondary_chronograph_value {
            app_state.set_secondary_chronograph_value(secondary_chronograph);
        }
        if let Some(readout) = self.state.readout_value {
            app_state.set_readout_value(readout);
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
                    }
                    WindowEvent::Resized(new_size) => {
                        fb_width = new_size.width as usize;
                        fb_height = new_size.height as usize;
                        let _ = pixels.resize_buffer(new_size.width, new_size.height);
                        let _ = pixels.resize_surface(new_size.width, new_size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(ref receiver) = receiver {
                            app_state.update_with_commands(receiver);
                        } else {
                            app_state.update();
                        }

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
            secondary_value: None,
            chronograph_value: None,
            secondary_chronograph_value: None,
            readout_value: None,
        };

        Self { config, state }
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
        cx: i32,
        cy: i32,
        r: i32,
        thickness: i32,
        start_angle: f64,
        arc_span: f64,
        color: (u8, u8, u8),
    },
    HighlightBand {
        cx: i32,
        cy: i32,
        r: i32,
        start_angle: f64,
        end_angle: f64,
        inner_radius: f64,
        outer_radius: f64,
    },
    Tick {
        cx: i32,
        cy: i32,
        r: i32,
        angle: f64,
        length: i32,
        thickness: f32,
        color: (u8, u8, u8),
    },
    Text {
        x: i32,
        y: i32,
        text: String,
        font_size: f32,
        color: (u8, u8, u8),
    },
    CurvedText {
        cx: i32,
        cy: i32,
        radius: f64,
        text: String,
        font_size: f32,
        arc_span: f64,
        start_angle: f64,
        color: (u8, u8, u8),
    },
    NeedleLine {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        thickness: f32,
        tapered: bool,
        color: (u8, u8, u8),
    },
    Circle {
        cx: i32,
        cy: i32,
        radius: i32,
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
                DrawCommand::Arc {
                    cx,
                    cy,
                    r,
                    thickness,
                    start_angle,
                    arc_span,
                    color,
                } => {
                    render_arc_immediate(
                        canvas,
                        *cx,
                        *cy,
                        *r,
                        *thickness,
                        *start_angle,
                        *arc_span,
                        *color,
                    );
                }
                DrawCommand::HighlightBand {
                    cx,
                    cy,
                    r,
                    start_angle,
                    end_angle,
                    inner_radius,
                    outer_radius,
                } => {
                    render_highlight_band_immediate(
                        canvas,
                        *cx,
                        *cy,
                        *r,
                        *start_angle,
                        *end_angle,
                        *inner_radius,
                        *outer_radius,
                        config,
                    );
                }
                DrawCommand::Tick {
                    cx,
                    cy,
                    r,
                    angle,
                    length,
                    thickness,
                    color,
                } => {
                    let outer_x = *cx as f64 + angle.cos() * (*r as f64 - 1.0);
                    let outer_y = *cy as f64 + angle.sin() * (*r as f64 - 1.0);
                    let inner_x = *cx as f64 + angle.cos() * (*r as f64 - *length as f64);
                    let inner_y = *cy as f64 + angle.sin() * (*r as f64 - *length as f64);
                    draw_thick_line_aa(
                        canvas.frame,
                        canvas.width,
                        inner_x.round() as i32,
                        inner_y.round() as i32,
                        outer_x.round() as i32,
                        outer_y.round() as i32,
                        *thickness,
                        color.0,
                        color.1,
                        color.2,
                    );
                }
                DrawCommand::Text {
                    x,
                    y,
                    text,
                    font_size,
                    color,
                } => {
                    let font =
                        Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
                    let scale = Scale::uniform(*font_size);
                    draw_text(
                        canvas.frame,
                        canvas.width,
                        canvas.height,
                        *x,
                        *y,
                        text,
                        &font,
                        scale,
                        *color,
                    );
                }
                DrawCommand::CurvedText {
                    cx,
                    cy,
                    radius,
                    text,
                    font_size,
                    arc_span,
                    start_angle,
                    color,
                } => {
                    let font =
                        Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
                    let scale = Scale::uniform(*font_size);
                    draw_curved_text(
                        canvas,
                        *cx,
                        *cy,
                        *radius,
                        text,
                        &font,
                        scale,
                        *arc_span,
                        *start_angle,
                        *color,
                    );
                }
                DrawCommand::NeedleLine {
                    x0,
                    y0,
                    x1,
                    y1,
                    thickness,
                    tapered,
                    color,
                } => {
                    if *tapered {
                        draw_thick_line_tapered_aa(
                            canvas.frame,
                            canvas.width,
                            *x0,
                            *y0,
                            *x1,
                            *y1,
                            *thickness,
                            color.0,
                            color.1,
                            color.2,
                        );
                    } else {
                        draw_thick_line_aa(
                            canvas.frame,
                            canvas.width,
                            *x0,
                            *y0,
                            *x1,
                            *y1,
                            *thickness,
                            color.0,
                            color.1,
                            color.2,
                        );
                    }
                }
                DrawCommand::Circle {
                    cx,
                    cy,
                    radius,
                    color,
                } => {
                    draw_circle(
                        canvas.frame,
                        canvas.width,
                        *cx,
                        *cy,
                        *radius,
                        color.0,
                        color.1,
                        color.2,
                    );
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
        Self {
            frame,
            width,
            height,
        }
    }

    fn clear(&mut self, color: (u8, u8, u8)) {
        for chunk in self.frame.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[color.0, color.1, color.2, 0xff]);
        }
    }
}

struct AppState {
    needle1: Option<Needle>,
    needle2: Option<Needle>,
    chronograph: Option<Needle>,
    secondary_chronograph: Option<Needle>,
    readout_value: Option<f64>,
    min_value: f64,
    max_value: f64,
    chronograph_range: (f64, f64),
    secondary_chronograph_range: (f64, f64),
    highlight_bounds: Option<(f64, f64)>,
}

impl AppState {
    fn new(min_value: f64, max_value: f64) -> Self {
        Self {
            needle1: None,
            needle2: None,
            chronograph: None,
            secondary_chronograph: None,
            readout_value: None,
            min_value,
            max_value,
            chronograph_range: (0.0, 60.0),
            secondary_chronograph_range: (0.0, 60.0),
            highlight_bounds: None,
        }
    }

    fn set_primary_value(&mut self, value: f64) {
        if self.needle1.is_none() {
            self.needle1 = Some(Needle::new());
        }
        if let Some(ref mut needle) = self.needle1 {
            needle.set_target_pos(
                ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0),
            );
        }
    }

    fn set_secondary_value(&mut self, value: f64) {
        if self.needle2.is_none() {
            self.needle2 = Some(Needle::new());
        }
        if let Some(ref mut needle) = self.needle2 {
            needle.set_target_pos(
                ((value - self.min_value) / (self.max_value - self.min_value)).clamp(0.0, 1.0),
            );
        }
    }

    fn set_chronograph_value(&mut self, value: f64) {
        if self.chronograph.is_none() {
            self.chronograph = Some(Needle::new());
        }
        if let Some(ref mut needle) = self.chronograph {
            let target_pos = ((value - self.chronograph_range.0)
                / (self.chronograph_range.1 - self.chronograph_range.0))
                .clamp(0.0, 1.0);
            needle.set_target_pos(target_pos);
        }
    }

    fn set_secondary_chronograph_value(&mut self, value: f64) {
        if self.secondary_chronograph.is_none() {
            self.secondary_chronograph = Some(Needle::new());
        }
        if let Some(ref mut needle) = self.secondary_chronograph {
            let target_pos = ((value - self.secondary_chronograph_range.0)
                / (self.secondary_chronograph_range.1 - self.secondary_chronograph_range.0))
                .clamp(0.0, 1.0);
            needle.set_target_pos(target_pos);
        }
    }

    fn set_readout_value(&mut self, value: f64) {
        self.readout_value = Some(value);
    }

    fn set_highlight_bounds(&mut self, lower: f64, upper: f64) {
        let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
        self.highlight_bounds = Some((min_bound, max_bound));
    }

    fn update(&mut self) {
        [
            &mut self.needle1,
            &mut self.needle2,
            &mut self.chronograph,
            &mut self.secondary_chronograph,
        ]
        .iter_mut()
        .filter_map(|n| n.as_mut())
        .for_each(|n| n.update_position());
    }

    fn update_with_commands(&mut self, receiver: &Receiver<InstrumentCommand>) {
        // Try to get the latest command without blocking
        while let Ok(command) = receiver.try_recv() {
            match command {
                InstrumentCommand::SetPrimaryNeedle(value) => {
                    self.set_primary_value(value);
                }
                InstrumentCommand::SetSecondaryNeedle(value) => {
                    self.set_secondary_value(value);
                }
                InstrumentCommand::SetReadout(value) => {
                    self.set_readout_value(value);
                }
                InstrumentCommand::SetHighlightBounds(lower, upper) => {
                    self.set_highlight_bounds(lower, upper);
                }
                InstrumentCommand::SetBothNeedles(primary, secondary) => {
                    self.set_primary_value(primary);
                    self.set_secondary_value(secondary);
                }
                InstrumentCommand::SetChronograph(value) => {
                    self.set_chronograph_value(value);
                }
                InstrumentCommand::SetSecondaryChronograph(value) => {
                    self.set_secondary_chronograph_value(value);
                }
                InstrumentCommand::SetAllNeedles(
                    primary,
                    secondary,
                    chronograph,
                    secondary_chronograph,
                ) => {
                    self.set_primary_value(primary);
                    self.set_secondary_value(secondary);
                    self.set_chronograph_value(chronograph);
                    self.set_secondary_chronograph_value(secondary_chronograph);
                }
                InstrumentCommand::SetBothChronographs(chronograph, secondary_chronograph) => {
                    self.set_chronograph_value(chronograph);
                    self.set_secondary_chronograph_value(secondary_chronograph);
                }
            }
        }

        self.update();
    }

    fn is_out_of_range(&self) -> bool {
        // Check if primary needle value is out of range
        if let Some(ref needle) = self.needle1 {
            let value = self.min_value + needle.pos * (self.max_value - self.min_value);
            if value < self.min_value || value > self.max_value {
                return true;
            }
        }
        // Check if secondary needle value is out of range
        if let Some(ref needle) = self.needle2 {
            let value = self.min_value + needle.pos * (self.max_value - self.min_value);
            if value < self.min_value || value > self.max_value {
                return true;
            }
        }
        false
    }

    fn set_highlight_override(&mut self, lower: f64, upper: f64) {
        let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
        self.highlight_bounds = Some((min_bound, max_bound));
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
        Self {
            cx,
            cy,
            r,
            thickness: config.dial_thickness,
            arc_span,
            start_angle,
        }
    }

    fn new_chronograph(width: usize, height: usize, config: &InstrumentConfig) -> Self {
        // Create a smaller dial for the chronograph
        let r = ((width.min(height) as f64) / config.chronograph_dial_size) as i32
            - config.chronograph_dial_margin;
        let cx = width as i32 / 2; // Center horizontally
        let cy = r + config.chronograph_dial_margin + config.chronograph_dial_shift; // Position in top middle
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self {
            cx,
            cy,
            r,
            thickness: config.chronograph_dial_thickness,
            arc_span,
            start_angle,
        }
    }

    fn new_secondary_chronograph(width: usize, height: usize, config: &InstrumentConfig) -> Self {
        // Create a smaller dial for the secondary chronograph
        let r = ((width.min(height) as f64) / config.secondary_chronograph_dial_size) as i32
            - config.secondary_chronograph_dial_margin;
        let cx = width as i32 / 2; // Center horizontally
        let cy = (height as i32 / 2) + config.secondary_chronograph_dial_shift; // Position below center
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self {
            cx,
            cy,
            r,
            thickness: config.secondary_chronograph_dial_thickness,
            arc_span,
            start_angle,
        }
    }
}

struct Needle {
    pos: f64, // Normalized [0,1]
    target_pos: f64,
}

impl Needle {
    fn new() -> Self {
        Self {
            pos: 0.5,
            target_pos: 0.5,
        }
    }

    fn set_target_pos(&mut self, target: f64) {
        self.target_pos = target.clamp(0.0, 1.0);
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
    let mut scene = Scene::new(canvas.width, canvas.height);
    scene.add_command(DrawCommand::Clear((0xff, 0xff, 0xff)));

    let dial = Dial::new(canvas.width, canvas.height, config);
    let is_out_of_range = state.is_out_of_range();
    let base_color = if is_out_of_range {
        (0xff, 0x00, 0x00)
    } else {
        (0x00, 0x00, 0x00)
    };
    let range = (state.min_value, state.max_value);

    // Add highlight band if needed
    if let Some(highlight) = state.highlight_bounds {
        let (hl_start, hl_end) = highlight;
        let (norm_hl_start, norm_hl_end) = (
            ((hl_start - range.0) / (range.1 - range.0)).clamp(0.0, 1.0),
            ((hl_end - range.0) / (range.1 - range.0)).clamp(0.0, 1.0),
        );
        scene.add_command(DrawCommand::HighlightBand {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            start_angle: dial.start_angle + dial.arc_span * norm_hl_start,
            end_angle: dial.start_angle + dial.arc_span * norm_hl_end,
            inner_radius: config.highlight_band_width as f64,
            outer_radius: 0.0,
        });
    }

    // Main dial with ticks and labels
    add_dial_with_ticks(
        &mut scene,
        &dial,
        range,
        config.ticks_count,
        config.major_tick_length,
        config.major_tick_thickness,
        config.minor_tick_thickness,
        config.minor_ticks_per_interval,
        config.minor_tick_length,
        config.dial_numbers_font_size,
        config.dial_ticks_to_numbers_distance,
        base_color,
    );

    // Curved text
    scene.add_command(DrawCommand::CurvedText {
        cx: dial.cx,
        cy: dial.cy,
        radius: dial.r as f64 + config.curved_text_radius_offset,
        text: config.curved_text.to_string(),
        font_size: config.curved_text_font_size,
        arc_span: config.curved_text_arc_span,
        start_angle: config.curved_text_angle,
        color: base_color,
    });

    // Needles
    if let Some(ref needle) = state.needle1 {
        let color = if is_out_of_range {
            (0xff, 0x00, 0x00)
        } else {
            (0x00, 0x00, 0x00)
        };
        add_needle(
            &mut scene,
            &dial,
            needle,
            color,
            config.needle_length_factor,
            config.needle_width,
            config.needle_back_length,
            config.dot_radius,
        );
    }
    if let Some(ref needle) = state.needle2 {
        let color = if is_out_of_range {
            (0xff, 0x00, 0x00)
        } else {
            (0x00, 0x7f, 0xff)
        };
        add_needle(
            &mut scene,
            &dial,
            needle,
            color,
            config.needle_length_factor,
            config.needle_width,
            config.needle_back_length,
            config.dot_radius,
        );
    }

    // Chronograph
    if let Some(ref needle) = state.chronograph {
        let color = if is_out_of_range {
            (0xff, 0x00, 0x00)
        } else {
            (0xff, 0x80, 0x00)
        };
        let chrono_dial = Dial::new_chronograph(canvas.width, canvas.height, config);
        add_dial_with_ticks(
            &mut scene,
            &chrono_dial,
            state.chronograph_range,
            config.chronograph_ticks_count,
            config.chronograph_tick_length,
            config.chronograph_major_tick_thickness,
            config.chronograph_minor_tick_thickness,
            config.chronograph_minor_ticks_per_interval,
            config.chronograph_minor_tick_length,
            config.chronograph_dial_numbers_font_size,
            config.chronograph_dial_ticks_to_numbers_distance,
            (0x00, 0x00, 0x00),
        );
        add_needle(
            &mut scene,
            &chrono_dial,
            needle,
            color,
            config.chronograph_needle_length_factor,
            config.chronograph_needle_width,
            config.chronograph_needle_back_length,
            config.chronograph_dial_dot_radius,
        );
    }

    // Secondary chronograph
    if let Some(ref needle) = state.secondary_chronograph {
        let color = if is_out_of_range {
            (0xff, 0x00, 0x00)
        } else {
            (0x00, 0x80, 0xff)
        };
        let sec_chrono_dial = Dial::new_secondary_chronograph(canvas.width, canvas.height, config);
        add_dial_with_ticks(
            &mut scene,
            &sec_chrono_dial,
            state.secondary_chronograph_range,
            config.secondary_chronograph_ticks_count,
            config.secondary_chronograph_tick_length,
            config.secondary_chronograph_major_tick_thickness,
            config.secondary_chronograph_minor_tick_thickness,
            config.secondary_chronograph_minor_ticks_per_interval,
            config.secondary_chronograph_minor_tick_length,
            config.secondary_chronograph_dial_numbers_font_size,
            config.secondary_chronograph_dial_ticks_to_numbers_distance,
            (0x00, 0x00, 0x00),
        );
        add_needle(
            &mut scene,
            &sec_chrono_dial,
            needle,
            color,
            config.secondary_chronograph_needle_length_factor,
            config.secondary_chronograph_needle_width,
            config.secondary_chronograph_needle_back_length,
            config.secondary_chronograph_dial_dot_radius,
        );
    }

    // Readout
    if let Some(value) = state.readout_value {
        let (value_int, value_frac) = (
            value.trunc() as i32,
            ((value.fract() * 1000.0).round() as u32).min(999),
        );
        let (label_x, label_y) = (
            (canvas.width as f64 * config.readout_x_factor) as i32,
            (canvas.height as f64 * config.readout_y_factor) as i32,
        );
        let value_str = format!("{}", value_int);
        scene.add_command(DrawCommand::Text {
            x: label_x,
            y: label_y,
            text: value_str.clone(),
            font_size: config.readout_big_font_size,
            color: base_color,
        });

        let font = Font::try_from_vec(config.font_data.to_vec()).expect("Error loading font");
        let int_width = calculate_text_width(
            &value_str,
            &font,
            Scale::uniform(config.readout_big_font_size),
        );
        let (frac_x, frac_y) = (label_x + int_width / 2 + 28, label_y + 2);
        scene.add_command(DrawCommand::Text {
            x: frac_x,
            y: frac_y,
            text: format!("{:03}", value_frac),
            font_size: config.readout_small_font_size,
            color: base_color,
        });

        // Readout box
        let (box_padding, box_thickness) = (
            config.readout_box_padding,
            config.readout_box_thickness as f32,
        );
        let font_size = (config.readout_big_font_size / 11.0) as i32;
        let (box_left, box_top, box_right, box_bottom) = (
            label_x - box_padding - font_size * value_str.len() as i32,
            label_y - box_padding,
            frac_x + box_padding + 5,
            frac_y + box_padding,
        );
        for (x0, y0, x1, y1) in [
            (box_left, box_top, box_right, box_top),
            (box_left, box_bottom, box_right, box_bottom),
            (box_left, box_top, box_left, box_bottom),
            (box_right, box_top, box_right, box_bottom),
        ] {
            scene.add_command(DrawCommand::NeedleLine {
                x0,
                y0,
                x1,
                y1,
                thickness: box_thickness,
                tapered: false,
                color: base_color,
            });
        }
    }

    // Warning indicator
    if is_out_of_range {
        scene.add_command(DrawCommand::Text {
            x: dial.cx,
            y: dial.cy - (dial.r / 4),
            text: "!".to_string(),
            font_size: config.exclamation_mark_size,
            color: (0xff, 0x00, 0x00),
        });
    }

    scene.render(canvas, config);
}

// Helper functions to reduce repetitive rendering code
fn add_dial_with_ticks(
    scene: &mut Scene,
    dial: &Dial,
    range: (f64, f64),
    ticks_count: usize,
    tick_length: i32,
    major_tick_thickness: f32,
    minor_tick_thickness: f32,
    minor_ticks_per_interval: usize,
    minor_tick_length: i32,
    font_size: f32,
    ticks_to_numbers_distance: f64,
    dial_color: (u8, u8, u8),
) {
    scene.add_command(DrawCommand::Arc {
        cx: dial.cx,
        cy: dial.cy,
        r: dial.r,
        thickness: dial.thickness,
        start_angle: dial.start_angle,
        arc_span: dial.arc_span,
        color: dial_color,
    });
    for i in 0..ticks_count {
        let t = i as f64 / (ticks_count as f64 - 1.0);
        let angle = dial.start_angle + dial.arc_span * t;
        scene.add_command(DrawCommand::Tick {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            angle,
            length: tick_length,
            thickness: major_tick_thickness,
            color: dial_color,
        });
        if i < ticks_count - 1 {
            for j in 1..=minor_ticks_per_interval {
                let minor_angle = dial.start_angle
                    + dial.arc_span
                        * (t + (j as f64
                            / (minor_ticks_per_interval as f64 * (ticks_count as f64 - 1.0))));
                scene.add_command(DrawCommand::Tick {
                    cx: dial.cx,
                    cy: dial.cy,
                    r: dial.r,
                    angle: minor_angle,
                    length: minor_tick_length,
                    thickness: minor_tick_thickness,
                    color: dial_color,
                });
            }
        }
        let label_radius = dial.r as f64 - tick_length as f64 - ticks_to_numbers_distance;
        let (label_x, label_y) = (
            dial.cx as f64 + angle.cos() * label_radius,
            dial.cy as f64 + angle.sin() * label_radius,
        );
        scene.add_command(DrawCommand::Text {
            x: label_x as i32,
            y: label_y as i32,
            text: format!("{}", (range.0 + t * (range.1 - range.0)).round() as i64),
            font_size,
            color: dial_color,
        });
    }
}

fn add_needle(
    scene: &mut Scene,
    dial: &Dial,
    needle: &Needle,
    color: (u8, u8, u8),
    length_factor: f64,
    width: f32,
    back_length: f64,
    dot_radius: i32,
) {
    let angle = dial.start_angle + dial.arc_span * needle.pos;
    let (nx, ny) = (
        (dial.cx as f64 + angle.cos() * dial.r as f64 * length_factor) as i32,
        (dial.cy as f64 + angle.sin() * dial.r as f64 * length_factor) as i32,
    );
    let (back_x, back_y) = (
        (dial.cx as f64 - angle.cos() * back_length) as i32,
        (dial.cy as f64 - angle.sin() * back_length) as i32,
    );
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: nx,
        y1: ny,
        thickness: width,
        tapered: true,
        color,
    });
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: back_x,
        y1: back_y,
        thickness: width,
        tapered: false,
        color,
    });
    scene.add_command(DrawCommand::Circle {
        cx: dial.cx,
        cy: dial.cy,
        radius: dot_radius,
        color,
    });
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn calculate_text_width(text: &str, font: &Font, scale: Scale) -> i32 {
    use rusttype::{point, PositionedGlyph};
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let (min_x, max_x, _, _) = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).fold(
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
    if min_x < max_x {
        max_x - min_x
    } else {
        0
    }
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
        let dst = [
            frame[idx] as f32,
            frame[idx + 1] as f32,
            frame[idx + 2] as f32,
            frame[idx + 3] as f32,
        ];
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

fn draw_thick_line_aa(
    frame: &mut [u8],
    width: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: f32,
    r: u8,
    g: u8,
    b: u8,
) {
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

fn draw_thick_line_tapered_aa(
    frame: &mut [u8],
    width: usize,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    thickness: f32,
    r: u8,
    g: u8,
    b: u8,
) {
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

fn draw_text(
    frame: &mut [u8],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    text: &str,
    font: &rusttype::Font,
    scale: rusttype::Scale,
    color: (u8, u8, u8),
) {
    use rusttype::{point, PositionedGlyph};
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<PositionedGlyph> = font
        .layout(text, scale, point(0.0, 0.0 + v_metrics.ascent))
        .collect();
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
                    set_pixel(
                        frame,
                        width,
                        px as usize,
                        py as usize,
                        color.0,
                        color.1,
                        color.2,
                        v as f32,
                    );
                }
            });
        }
    }
}

fn draw_curved_text(
    canvas: &mut Canvas,
    cx: i32,
    cy: i32,
    radius: f64,
    text: &str,
    font: &rusttype::Font,
    scale: rusttype::Scale,
    arc_span: f64,
    center_angle: f64,
    color: (u8, u8, u8),
) {
    use rusttype::{point, PositionedGlyph};

    // Create glyphs for the text to calculate individual character positions
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<PositionedGlyph> = font
        .layout(text, scale, point(0.0, 0.0 + v_metrics.ascent))
        .collect();

    if glyphs.is_empty() {
        return;
    }

    // Calculate total text width by examining glyph positions
    let total_width = if let (Some(first), Some(last)) = (glyphs.first(), glyphs.last()) {
        (last.position().x - first.position().x + last.unpositioned().h_metrics().advance_width)
            as f64
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

fn draw_rotated_glyph_improved(
    canvas: &mut Canvas,
    glyph: &rusttype::PositionedGlyph,
    center_x: f64,
    center_y: f64,
    rotation: f64,
    color: (u8, u8, u8),
) {
    if let Some(bb) = glyph.pixel_bounding_box() {
        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        // Calculate glyph center offset
        let glyph_center_x = (bb.min.x + bb.max.x) as f64 / 2.0;
        let glyph_center_y = (bb.min.y + bb.max.y) as f64 / 2.0;

        // Draw each pixel of the glyph with sub-pixel accuracy
        glyph.draw(|gx, gy, v| {
            if v > 0.001 {
                // Lower threshold for better coverage
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
            if final_alpha > 0.001 {
                // Lower threshold for better coverage
                set_pixel(
                    canvas.frame,
                    canvas.width,
                    *px as usize,
                    *py as usize,
                    color.0,
                    color.1,
                    color.2,
                    final_alpha,
                );
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
                if px >= 0
                    && py >= 0
                    && (px as usize) < width
                    && (py as usize) < frame.len() / (width * 4)
                {
                    set_pixel(frame, width, px as usize, py as usize, r, g, b, aa as f32);
                }
            }
        }
    }
}

fn render_arc_immediate(
    canvas: &mut Canvas,
    cx: i32,
    cy: i32,
    r: i32,
    thickness: i32,
    start_angle: f64,
    arc_span: f64,
    color: (u8, u8, u8),
) {
    let end_angle = start_angle + arc_span;
    let mut start_angle = start_angle;
    let mut end_angle = end_angle;
    if start_angle < 0.0 {
        start_angle += 2.0 * std::f64::consts::PI;
    }
    if end_angle >= 2.0 * std::f64::consts::PI {
        end_angle -= 2.0 * std::f64::consts::PI;
    }

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
            if start < 0.0 {
                start += 2.0 * std::f64::consts::PI;
            }
            if end < 0.0 {
                end += 2.0 * std::f64::consts::PI;
            }
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
                    set_pixel(
                        canvas.frame,
                        canvas.width,
                        x as usize,
                        y as usize,
                        color.0,
                        color.1,
                        color.2,
                        aa as f32,
                    );
                }
            }
        }
    }
}

fn render_highlight_band_immediate(
    canvas: &mut Canvas,
    cx: i32,
    cy: i32,
    r: i32,
    start_angle: f64,
    end_angle: f64,
    inner_radius: f64,
    outer_radius: f64,
    config: &InstrumentConfig,
) {
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
                    angular_alpha = 1.0
                        - ((start_angle - angle).min(config.highlight_band_edge_softness)
                            / config.highlight_band_edge_softness);
                } else if angle > end_angle {
                    angular_alpha = 1.0
                        - ((angle - end_angle).min(config.highlight_band_edge_softness)
                            / config.highlight_band_edge_softness);
                }
                if angle < start_angle || angle > end_angle {
                    angular_alpha = angular_alpha.max(0.0);
                }
            } else {
                // Wrap case: start > end (crosses 0 degrees)
                if angle < end_angle {
                    // Close to end edge
                    angular_alpha = 1.0
                        - ((end_angle - angle).min(config.highlight_band_edge_softness)
                            / config.highlight_band_edge_softness)
                            .max(0.0);
                } else if angle > start_angle {
                    // Close to start edge
                    angular_alpha = 1.0
                        - ((angle - start_angle).min(config.highlight_band_edge_softness)
                            / config.highlight_band_edge_softness)
                            .max(0.0);
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
                    angular_alpha = 1.0
                        - (min_dist.min(config.highlight_band_edge_softness)
                            / config.highlight_band_edge_softness);
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

            let final_alpha =
                (angular_alpha * radial_alpha * config.highlight_band_alpha).clamp(0.0, 1.0);

            if final_alpha > 0.01 {
                let color = config.highlight_band_color.as_tuple();
                set_pixel(
                    canvas.frame,
                    canvas.width,
                    x as usize,
                    y as usize,
                    color.0,
                    color.1,
                    color.2,
                    final_alpha as f32,
                );
            }
        }
    }
}

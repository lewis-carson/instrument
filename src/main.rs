mod config;

use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::process;
use std::time::Instant;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use rusttype::{Font, Scale};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::io::{self, BufRead};
use std::env;
use std::collections::HashMap;

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
    width: usize,
    height: usize,
}

impl Scene {
    fn new(width: usize, height: usize) -> Self {
        Self {
            commands: Vec::new(),
            width,
            height,
        }
    }

    fn clear(&mut self) {
        self.commands.clear();
    }

    fn add_command(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    fn render(&self, canvas: &mut Canvas) {
        for command in &self.commands {
            match command {
                DrawCommand::Clear(color) => {
                    canvas.clear(*color);
                }
                DrawCommand::Arc { cx, cy, r, thickness, start_angle, arc_span, color } => {
                    render_arc_immediate(canvas, *cx, *cy, *r, *thickness, *start_angle, *arc_span, *color);
                }
                DrawCommand::HighlightBand { cx, cy, r, start_angle, end_angle, inner_radius, outer_radius } => {
                    render_highlight_band_immediate(canvas, *cx, *cy, *r, *start_angle, *end_angle, *inner_radius, *outer_radius);
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
                    let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");
                    let scale = Scale::uniform(*font_size);
                    draw_text(canvas.frame, canvas.width, canvas.height, *x, *y, text, &font, scale, *color);
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

    fn set_highlight_range(&mut self, lower: f64, upper: f64) {
        let (min_bound, max_bound) = (lower.min(upper), lower.max(upper));
        if let Some(ref mut bounds) = self.highlight_bounds {
            bounds.set_target_bounds(min_bound, max_bound);
        } else {
            self.highlight_bounds = Some(HighlightBounds::new(min_bound, max_bound));
        }
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
    fn new(width: usize, height: usize) -> Self {
        let cx = width as i32 / 2;
        let cy = height as i32 / 2;
        let r = (width.min(height) as i32) / 2 - config::DIAL_MARGIN;
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self { cx, cy, r, thickness: config::DIAL_THICKNESS, arc_span, start_angle }
    }

    fn draw_with_highlight(&self, canvas: &mut Canvas, range: (f64, f64), color: (u8, u8, u8), highlight_range: Option<(f64, f64)>) {
        if let Some(highlight) = highlight_range {
            self.draw_highlight_band(canvas, range, highlight);
        }
        self.draw_arc(canvas, color);
        self.draw_ticks(canvas, range, color);
    }

    fn draw_arc(&self, canvas: &mut Canvas, color: (u8, u8, u8)) {
        let end_angle = self.start_angle + self.arc_span;
        let mut start_angle = self.start_angle;
        let mut end_angle = end_angle;
        if start_angle < 0.0 { start_angle += 2.0 * std::f64::consts::PI; }
        if end_angle >= 2.0 * std::f64::consts::PI { end_angle -= 2.0 * std::f64::consts::PI; }
        
        for y in 0..canvas.height as i32 {
            for x in 0..canvas.width as i32 {
                let dx = x - self.cx;
                let dy = y - self.cy;
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
                    let aa = if dist > self.r as f64 {
                        1.0 - (dist - self.r as f64).min(1.0)
                    } else if dist < (self.r - self.thickness) as f64 {
                        1.0 - ((self.r - self.thickness) as f64 - dist).min(1.0)
                    } else {
                        1.0
                    };
                    if dist >= (self.r - self.thickness - 1) as f64 && dist <= (self.r + 1) as f64 && aa > 0.0 {
                        set_pixel(canvas.frame, canvas.width, x as usize, y as usize, color.0, color.1, color.2, aa as f32);
                    }
                }
            }
        }
    }

    fn draw_highlight_band(&self, canvas: &mut Canvas, range: (f64, f64), highlight_range: (f64, f64)) {
        let (hl_start, hl_end) = highlight_range;
        let (r_start, r_end) = range;

        // Convert to normalized [0,1] range
        let norm_hl_start = ((hl_start - r_start) / (r_end - r_start)).clamp(0.0, 1.0);
        let norm_hl_end = ((hl_end - r_start) / (r_end - r_start)).clamp(0.0, 1.0);

        // Calculate angles for the highlight band
        let start_angle = self.start_angle + self.arc_span * norm_hl_start;
        let end_angle = self.start_angle + self.arc_span * norm_hl_end;

        // Draw the highlight band as a thick arc
        let band_inner_radius = (self.r - config::HIGHLIGHT_BAND_WIDTH) as f64;
        let band_outer_radius = self.r as f64;
        
        for y in 0..canvas.height as i32 {
            for x in 0..canvas.width as i32 {
                let dx = x - self.cx;
                let dy = y - self.cy;
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
                        angular_alpha = 1.0 - ((start_angle - angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
                    } else if angle > end_angle {
                        angular_alpha = 1.0 - ((angle - end_angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
                    }
                    if angle < start_angle || angle > end_angle {
                        angular_alpha = angular_alpha.max(0.0);
                    }
                } else {
                    // Wrap case: start > end (crosses 0 degrees)
                    if angle < end_angle {
                        // Close to end edge
                        angular_alpha = 1.0 - ((end_angle - angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS).max(0.0);
                    } else if angle > start_angle {
                        // Close to start edge  
                        angular_alpha = 1.0 - ((angle - start_angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS).max(0.0);
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
                        angular_alpha = 1.0 - (min_dist.min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
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
                
                let final_alpha = (angular_alpha * radial_alpha * config::HIGHLIGHT_ALPHA).clamp(0.0, 1.0);
                
                if final_alpha > 0.01 {
                    set_pixel(canvas.frame, canvas.width, x as usize, y as usize, 
                            config::HIGHLIGHT_COLOR.0, config::HIGHLIGHT_COLOR.1, config::HIGHLIGHT_COLOR.2, final_alpha as f32);
                }
            }
        }
    }

    fn draw_ticks(&self, canvas: &mut Canvas, range: (f64, f64), color: (u8, u8, u8)) {
        let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");
        let dial_numbers_scale = Scale::uniform(config::DIAL_NUMBERS_FONT_SIZE);
        
        for i in 0..config::NUM_TICKS {
            let t = i as f64 / (config::NUM_TICKS as f64 - 1.0);
            let angle = self.start_angle + self.arc_span * t;
            
            // Major tick
            self.draw_single_tick(canvas, angle, config::TICK_LENGTH, config::TICK_THICKNESS, color);
            
            // Minor ticks
            if i < config::NUM_TICKS - 1 {
                for j in 1..=config::MINOR_TICKS_PER_INTERVAL {
                    let minor_t = t + (j as f64 / (config::MINOR_TICKS_PER_INTERVAL as f64 * (config::NUM_TICKS as f64 - 1.0)));
                    let minor_angle = self.start_angle + self.arc_span * minor_t;
                    self.draw_single_tick(canvas, minor_angle, config::MINOR_TICK_LENGTH, config::MINOR_TICK_THICKNESS, color);
                }
            }
            
            // Number labels
            let label_radius = self.r as f64 - config::TICK_LENGTH as f64 - config::TICKS_TO_NUMBERS_DISTANCE;
            let label_x = self.cx as f64 + angle.cos() * label_radius;
            let label_y = self.cy as f64 + angle.sin() * label_radius;
            let value = range.0 + t * (range.1 - range.0);
            let value_str = format!("{}", value.round() as i64);
            draw_text(canvas.frame, canvas.width, canvas.height, label_x as i32, label_y as i32, &value_str, &font, dial_numbers_scale, color);
        }
    }

    fn draw_single_tick(&self, canvas: &mut Canvas, angle: f64, length: i32, thickness: f32, color: (u8, u8, u8)) {
        let outer_x = self.cx as f64 + angle.cos() * (self.r as f64 - 1.0);
        let outer_y = self.cy as f64 + angle.sin() * (self.r as f64 - 1.0);
        let inner_x = self.cx as f64 + angle.cos() * (self.r as f64 - length as f64);
        let inner_y = self.cy as f64 + angle.sin() * (self.r as f64 - length as f64);
        draw_thick_line_aa(
            canvas.frame,
            canvas.width,
            inner_x.round() as i32,
            inner_y.round() as i32,
            outer_x.round() as i32,
            outer_y.round() as i32,
            thickness,
            color.0, color.1, color.2
        );
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

    fn draw(&self, canvas: &mut Canvas, dial: &Dial, color: (u8, u8, u8)) {
        let angle = dial.start_angle + dial.arc_span * self.pos;
        let needle_length = dial.r as f64 * config::NEEDLE_LENGTH_FACTOR;
        let nx = (dial.cx as f64 + angle.cos() * needle_length) as i32;
        let ny = (dial.cy as f64 + angle.sin() * needle_length) as i32;

        draw_thick_line_tapered_aa(canvas.frame, canvas.width, dial.cx, dial.cy, nx, ny, config::NEEDLE_WIDTH, color.0, color.1, color.2);

        // Draw the needle's back extension
        let back_length = config::NEEDLE_BACK_LENGTH;
        let back_x = (dial.cx as f64 - angle.cos() * back_length) as i32;
        let back_y = (dial.cy as f64 - angle.sin() * back_length) as i32;
        draw_thick_line_aa(canvas.frame, canvas.width, dial.cx, dial.cy, back_x, back_y, config::NEEDLE_WIDTH, color.0, color.1, color.2);

        // Draw the needle's crossbar or dot
        match config::DEFAULT_CROSSBAR_TYPE {
            config::CrossbarType::BAR => {
                let crossbar_length = config::NEEDLE_CROSSBAR_LENGTH;
                let crossbar_thickness = config::NEEDLE_CROSSBAR_THICKNESS;
                let crossbar_angle = angle + std::f64::consts::FRAC_PI_2; // Perpendicular to the needle
                let crossbar_x1 = (dial.cx as f64 + crossbar_angle.cos() * (crossbar_length / 2.0)) as i32;
                let crossbar_y1 = (dial.cy as f64 + crossbar_angle.sin() * (crossbar_length / 2.0)) as i32;
                let crossbar_x2 = (dial.cx as f64 - crossbar_angle.cos() * (crossbar_length / 2.0)) as i32;
                let crossbar_y2 = (dial.cy as f64 - crossbar_angle.sin() * (crossbar_length / 2.0)) as i32;
                draw_thick_line_aa(canvas.frame, canvas.width, crossbar_x1, crossbar_y1, crossbar_x2, crossbar_y2, crossbar_thickness, color.0, color.1, color.2);
            }
            config::CrossbarType::DOT => {
                let dot_radius = config::DOT_RADIUS as i32;
                draw_circle(canvas.frame, canvas.width, dial.cx, dial.cy, dot_radius, color.0, color.1, color.2);
            }
        }
    }
}

// ============================================================================
// RETAINED MODE BUILDER FUNCTIONS
// ============================================================================

fn build_dial_commands(scene: &mut Scene, dial: &Dial, range: (f64, f64), color: (u8, u8, u8), highlight_range: Option<(f64, f64)>) {
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
            inner_radius: config::HIGHLIGHT_BAND_WIDTH as f64,
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
    for i in 0..config::NUM_TICKS {
        let t = i as f64 / (config::NUM_TICKS as f64 - 1.0);
        let angle = dial.start_angle + dial.arc_span * t;
        
        // Major tick
        scene.add_command(DrawCommand::Tick {
            cx: dial.cx,
            cy: dial.cy,
            r: dial.r,
            angle,
            length: config::TICK_LENGTH,
            thickness: config::TICK_THICKNESS,
            color,
        });
        
        // Minor ticks
        if i < config::NUM_TICKS - 1 {
            for j in 1..=config::MINOR_TICKS_PER_INTERVAL {
                let minor_t = t + (j as f64 / (config::MINOR_TICKS_PER_INTERVAL as f64 * (config::NUM_TICKS as f64 - 1.0)));
                let minor_angle = dial.start_angle + dial.arc_span * minor_t;
                scene.add_command(DrawCommand::Tick {
                    cx: dial.cx,
                    cy: dial.cy,
                    r: dial.r,
                    angle: minor_angle,
                    length: config::MINOR_TICK_LENGTH,
                    thickness: config::MINOR_TICK_THICKNESS,
                    color,
                });
            }
        }
        
        // Number labels
        let label_radius = dial.r as f64 - config::TICK_LENGTH as f64 - config::TICKS_TO_NUMBERS_DISTANCE;
        let label_x = dial.cx as f64 + angle.cos() * label_radius;
        let label_y = dial.cy as f64 + angle.sin() * label_radius;
        let value = range.0 + t * (range.1 - range.0);
        let value_str = format!("{}", value.round() as i64);
        scene.add_command(DrawCommand::Text {
            x: label_x as i32,
            y: label_y as i32,
            text: value_str,
            font_size: config::DIAL_NUMBERS_FONT_SIZE,
            color,
        });
    }
}

fn build_needle_commands(scene: &mut Scene, needle: &Needle, dial: &Dial, color: (u8, u8, u8)) {
    let angle = dial.start_angle + dial.arc_span * needle.pos;
    let needle_length = dial.r as f64 * config::NEEDLE_LENGTH_FACTOR;
    let nx = (dial.cx as f64 + angle.cos() * needle_length) as i32;
    let ny = (dial.cy as f64 + angle.sin() * needle_length) as i32;

    // Main needle line
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: nx,
        y1: ny,
        thickness: config::NEEDLE_WIDTH,
        tapered: true,
        color,
    });

    // Draw the needle's back extension
    let back_length = config::NEEDLE_BACK_LENGTH;
    let back_x = (dial.cx as f64 - angle.cos() * back_length) as i32;
    let back_y = (dial.cy as f64 - angle.sin() * back_length) as i32;
    scene.add_command(DrawCommand::NeedleLine {
        x0: dial.cx,
        y0: dial.cy,
        x1: back_x,
        y1: back_y,
        thickness: config::NEEDLE_WIDTH,
        tapered: false,
        color,
    });

    // Draw the needle's crossbar or dot
    match config::DEFAULT_CROSSBAR_TYPE {
        config::CrossbarType::BAR => {
            let crossbar_length = config::NEEDLE_CROSSBAR_LENGTH;
            let crossbar_thickness = config::NEEDLE_CROSSBAR_THICKNESS;
            let crossbar_angle = angle + std::f64::consts::FRAC_PI_2; // Perpendicular to the needle
            let crossbar_x1 = (dial.cx as f64 + crossbar_angle.cos() * (crossbar_length / 2.0)) as i32;
            let crossbar_y1 = (dial.cy as f64 + crossbar_angle.sin() * (crossbar_length / 2.0)) as i32;
            let crossbar_x2 = (dial.cx as f64 - crossbar_angle.cos() * (crossbar_length / 2.0)) as i32;
            let crossbar_y2 = (dial.cy as f64 - crossbar_angle.sin() * (crossbar_length / 2.0)) as i32;
            scene.add_command(DrawCommand::NeedleLine {
                x0: crossbar_x1,
                y0: crossbar_y1,
                x1: crossbar_x2,
                y1: crossbar_y2,
                thickness: crossbar_thickness,
                tapered: false,
                color,
            });
        }
        config::CrossbarType::DOT => {
            let dot_radius = config::DOT_RADIUS as i32;
            scene.add_command(DrawCommand::Circle {
                cx: dial.cx,
                cy: dial.cy,
                radius: dot_radius,
                color,
            });
        }
    }
}

fn build_readout_commands(scene: &mut Scene, canvas: &Canvas, value: f64, color: (u8, u8, u8)) {
    let value_int = value.trunc() as i32;
    let value_frac = ((value.fract() * 1000.0).round() as u32).min(999);
    let label_x = (canvas.width as f64 * config::READOUT_X_FACTOR) as i32;
    let label_y = (canvas.height as f64 * config::READOUT_Y_FACTOR) as i32;

    // Integer part in big font
    let value_str = format!("{}", value_int);
    scene.add_command(DrawCommand::Text {
        x: label_x,
        y: label_y,
        text: value_str.clone(),
        font_size: config::READOUT_BIG_FONT_SIZE,
        color,
    });

    // Fractional part in smaller font
    let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");
    let scale_big = Scale::uniform(config::READOUT_BIG_FONT_SIZE);
    let int_width = calculate_text_width(&value_str, &font, scale_big);
    let frac_str = format!("{:03}", value_frac);
    let frac_x = label_x + int_width / 2 + 28;
    let frac_y = label_y + 2;
    scene.add_command(DrawCommand::Text {
        x: frac_x,
        y: frac_y,
        text: frac_str,
        font_size: config::READOUT_SMALL_FONT_SIZE,
        color,
    });

    // TODO: Add readout box drawing commands
    // For now, we'll keep the immediate mode box drawing in a separate helper
    build_readout_box_commands(scene, label_x, label_y, frac_x, frac_y, &value_str, color);
}

fn build_readout_box_commands(scene: &mut Scene, label_x: i32, label_y: i32, frac_x: i32, frac_y: i32, int_str: &str, color: (u8, u8, u8)) {
    let box_padding = config::READOUT_BOX_PADDING;
    let box_thickness = config::READOUT_BOX_THICKNESS;
    let font_size = (config::READOUT_BIG_FONT_SIZE / 11.0) as i32;
    
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

fn build_warning_commands(scene: &mut Scene, dial: &Dial) {
    let exclamation_x = dial.cx;
    let exclamation_y = dial.cy - (dial.r / 4);
    let exclamation_str = "!";
    let exclamation_color = (0xff, 0x00, 0x00);
    scene.add_command(DrawCommand::Text {
        x: exclamation_x,
        y: exclamation_y,
        text: exclamation_str.to_string(),
        font_size: config::EXCLAMATION_MARK_FONT_SIZE,
        color: exclamation_color,
    });
}

// ============================================================================
// APPLICATION LOGIC
// ============================================================================

fn parse_args() -> (f64, f64, String, Option<(f64, f64)>) {
    let mut min_value = 0.0;
    let mut max_value = 100.0;
    let mut window_title = "Instrument".to_string();
    let mut highlight_range = None;
    let mut args = env::args().peekable();
    
    while let Some(arg) = args.next() {
        if arg == "--range" {
            if let (Some(x), Some(y)) = (args.next(), args.next()) {
                if let (Ok(x), Ok(y)) = (x.parse::<f64>(), y.parse::<f64>()) {
                    min_value = x.min(y);
                    max_value = x.max(y);
                }
            }
        } else if arg == "--title" {
            if let Some(title) = args.next() {
                window_title = title;
            }
        } else if arg == "--highlight" {
            if let (Some(upper), Some(lower)) = (args.next(), args.next()) {
                if let (Ok(upper), Ok(lower)) = (upper.parse::<f64>(), lower.parse::<f64>()) {
                    highlight_range = Some((lower.min(upper), lower.max(upper)));
                }
            }
        }
    }
    (min_value, max_value, window_title, highlight_range)
}

fn spawn_input_thread() -> Receiver<HashMap<String, f64>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                let line = line.trim();
                
                // Check if line contains key-value pairs (has '=' character)
                if line.contains('=') {
                    let key_values = parse_key_value_line(line);
                    
                    if !key_values.is_empty() {
                        let _ = sender.send(key_values);
                    }
                } else {
                    // Try to parse as a single numeric value (backwards compatibility)
                    if let Ok(val) = line.parse::<f64>() {
                        let mut single_value_map = HashMap::new();
                        single_value_map.insert("needle1".to_string(), val);
                        single_value_map.insert("readout".to_string(), val);
                        let _ = sender.send(single_value_map);
                    }
                }
            }
        }
    });
    receiver
}

fn parse_key_value_line(line: &str) -> HashMap<String, f64> {
    let mut key_values = HashMap::new();
    
    // Split by spaces to get individual key=value pairs
    for pair in line.split_whitespace() {
        if let Some(eq_pos) = pair.find('=') {
            let key = pair[..eq_pos].to_string();
            let value_str = &pair[eq_pos + 1..];
            
            if let Ok(value) = value_str.parse::<f64>() {
                key_values.insert(key, value);
            }
        }
    }
    
    key_values
}

fn render_instrument(canvas: &mut Canvas, state: &AppState) {
    // Create a scene to build our drawing commands
    let mut scene = Scene::new(canvas.width, canvas.height);
    
    // Add clear command
    scene.add_command(DrawCommand::Clear((0xff, 0xff, 0xff)));
    
    let dial = Dial::new(canvas.width, canvas.height);
    let is_out_of_range = state.is_out_of_range();
    let base_color = if is_out_of_range { (0xff, 0x00, 0x00) } else { (0x00, 0x00, 0x00) };
    
    // Build dial drawing commands
    build_dial_commands(&mut scene, &dial, (state.min_value, state.max_value), base_color, state.highlight_bounds.as_ref().map(|b| b.get_bounds()));
    
    // Draw needle1 if it exists
    if let Some(ref needle) = state.needle1 {
        let needle_color = if is_out_of_range { 
            (0xff, 0x00, 0x00) 
        } else { 
            (0x00, 0x00, 0x00) // Black for needle1
        };
        build_needle_commands(&mut scene, needle, &dial, needle_color);
    }
    
    // Draw needle2 if it exists
    if let Some(ref needle) = state.needle2 {
        let needle_color = if is_out_of_range { 
            (0xff, 0x00, 0x00) 
        } else { 
            (0x00, 0x7f, 0xff) // Blue for needle2
        };
        build_needle_commands(&mut scene, needle, &dial, needle_color);
    }
    
    // Draw readout if readout value exists
    if let Some(&value) = state.current_values.get("readout") {
        build_readout_commands(&mut scene, canvas, value, base_color);
    }
    
    // Draw warning indicator
    if is_out_of_range {
        build_warning_commands(&mut scene, &dial);
    }
    
    // Render the complete scene
    scene.render(canvas);
}

fn draw_readout(canvas: &mut Canvas, value: f64, color: (u8, u8, u8)) {
    let value_int = value.trunc() as i32;
    let value_frac = ((value.fract() * 1000.0).round() as u32).min(999);
    let label_x = (canvas.width as f64 * config::READOUT_X_FACTOR) as i32;
    let label_y = (canvas.height as f64 * config::READOUT_Y_FACTOR) as i32;

    let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");

    // Draw integer part in big font
    let scale_big = Scale::uniform(config::READOUT_BIG_FONT_SIZE);
    let value_str = format!("{}", value_int);
    draw_text(canvas.frame, canvas.width, canvas.height, label_x, label_y, &value_str, &font, scale_big, color);

    // Draw fractional part in smaller font
    let int_str = format!("{}", value_int);
    let int_width = calculate_text_width(&int_str, &font, scale_big);
    let scale_small = Scale::uniform(config::READOUT_SMALL_FONT_SIZE);
    let frac_str = format!("{:03}", value_frac);
    let frac_x = label_x + int_width / 2 + 28;
    let frac_y = label_y + 2;
    draw_text(canvas.frame, canvas.width, canvas.height, frac_x, frac_y, &frac_str, &font, scale_small, color);

    // Draw box around readout
    draw_readout_box(canvas, label_x, label_y, frac_x, frac_y, &int_str, color);
}

fn calculate_text_width(text: &str, font: &Font, scale: Scale) -> i32 {
    use rusttype::{point, PositionedGlyph};
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let (min_x, max_x, _, _) = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y), bb| {
            (min_x.min(bb.min.x), max_x.max(bb.max.x), min_y, max_y)
        },
    );
    if min_x < max_x { max_x - min_x } else { 0 }
}

fn draw_readout_box(canvas: &mut Canvas, label_x: i32, label_y: i32, frac_x: i32, frac_y: i32, int_str: &str, color: (u8, u8, u8)) {
    let box_padding = config::READOUT_BOX_PADDING;
    let box_thickness = config::READOUT_BOX_THICKNESS;
    let font_size = (config::READOUT_BIG_FONT_SIZE / 11.0) as i32;
    
    let box_left = label_x - box_padding - font_size * int_str.len() as i32;
    let box_top = label_y - box_padding;
    let box_right = frac_x + box_padding + 5;
    let box_bottom = frac_y + box_padding;

    // Draw box lines
    draw_thick_line_aa(canvas.frame, canvas.width, box_left, box_top, box_right, box_top, box_thickness as f32, color.0, color.1, color.2);
    draw_thick_line_aa(canvas.frame, canvas.width, box_left, box_bottom, box_right, box_bottom, box_thickness as f32, color.0, color.1, color.2);
    draw_thick_line_aa(canvas.frame, canvas.width, box_left, box_top, box_left, box_bottom, box_thickness as f32, color.0, color.1, color.2);
    draw_thick_line_aa(canvas.frame, canvas.width, box_right, box_top, box_right, box_bottom, box_thickness as f32, color.0, color.1, color.2);
}

fn draw_warning(canvas: &mut Canvas, dial: &Dial) {
    let exclamation_x = dial.cx;
    let exclamation_y = dial.cy - (dial.r / 4);
    let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");
    let scale = Scale::uniform(config::EXCLAMATION_MARK_FONT_SIZE);
    let exclamation_str = "!";
    let exclamation_color = (0xff, 0x00, 0x00);
    draw_text(canvas.frame, canvas.width, canvas.height, exclamation_x, exclamation_y, exclamation_str, &font, scale, exclamation_color);
}

fn main() {
    // Parse command line arguments
    let (min_value, max_value, window_title, highlight_range) = parse_args();
    
    // Initialize application state
    let mut app_state = AppState::new(min_value, max_value);
    
    // Set highlight override if provided via command line
    if let Some((lower, upper)) = highlight_range {
        app_state.set_highlight_override(lower, upper);
    }
    
    // Spawn input thread and get receiver
    let receiver = spawn_input_thread();
    
    // Set up window and graphics
    let logical_width: usize = config::WINDOW_WIDTH;
    let logical_height: usize = config::WINDOW_HEIGHT;
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(&window_title)
        .with_inner_size(LogicalSize::new(logical_width as f64, logical_height as f64))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();
    let window = std::sync::Arc::new(window);
    let window_clone = window.clone();

    let mut fb_width = window.inner_size().width as usize;
    let mut fb_height = window.inner_size().height as usize;
    let mut pixels = {
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        Pixels::new(size.width, size.height, surface_texture).unwrap()
    };
    
    // Frame timing
    let target_fps = config::MAX_FRAMERATE;
    let frame_duration = std::time::Duration::from_secs_f64(1.0 / target_fps);
    let mut last_frame = Instant::now();

    let _ = event_loop.run(move |event, _window_target| {
        _window_target.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    process::exit(0);
                },
                WindowEvent::Resized(new_size) => {
                    fb_width = new_size.width as usize;
                    fb_height = new_size.height as usize;
                    let _ = pixels.resize_surface(new_size.width, new_size.height);
                }
                WindowEvent::ScaleFactorChanged { scale_factor: new_scale, mut inner_size_writer } => {
                    let size = ((400.0 * new_scale) as u32, (400.0 * new_scale) as u32);
                    fb_width = size.0 as usize;
                    fb_height = size.1 as usize;
                    let _ = inner_size_writer.request_inner_size(PhysicalSize::new(size.0, size.1));
                    let _ = pixels.resize_surface(size.0, size.1);
                }
                WindowEvent::RedrawRequested => {
                    // Update application state
                    app_state.update(&receiver);
                    
                    // Render frame
                    let frame = pixels.frame_mut();
                    let mut canvas = Canvas::new(frame, fb_width, fb_height);
                    render_instrument(&mut canvas, &app_state);
                    let _ = pixels.render();
                }
                _ => {}
            },
            Event::AboutToWait => {
                // Limit redraws to target frame rate
                let now = Instant::now();
                let elapsed = now.duration_since(last_frame);
                if elapsed < frame_duration {
                    std::thread::sleep(frame_duration - elapsed);
                }
                last_frame = Instant::now();
                window_clone.request_redraw();
            },
            _ => {}
        }
    });
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

fn lerp(current: f64, target: f64) -> f64 {
    current + (target - current) * config::LERP_FACTOR
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
                    set_pixel(frame, width, px as usize, py as usize, color.0, color.1, color.2, v as f32); // Set text color to black
                }
            });
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

fn render_highlight_band_immediate(canvas: &mut Canvas, cx: i32, cy: i32, r: i32, start_angle: f64, end_angle: f64, inner_radius: f64, outer_radius: f64) {
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
                    angular_alpha = 1.0 - ((start_angle - angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
                } else if angle > end_angle {
                    angular_alpha = 1.0 - ((angle - end_angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
                }
                if angle < start_angle || angle > end_angle {
                    angular_alpha = angular_alpha.max(0.0);
                }
            } else {
                // Wrap case: start > end (crosses 0 degrees)
                if angle < end_angle {
                    // Close to end edge
                    angular_alpha = 1.0 - ((end_angle - angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS).max(0.0);
                } else if angle > start_angle {
                    // Close to start edge  
                    angular_alpha = 1.0 - ((angle - start_angle).min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS).max(0.0);
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
                    angular_alpha = 1.0 - (min_dist.min(config::HIGHLIGHT_EDGE_SOFTNESS) / config::HIGHLIGHT_EDGE_SOFTNESS);
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
            
            let final_alpha = (angular_alpha * radial_alpha * config::HIGHLIGHT_ALPHA).clamp(0.0, 1.0);
            
            if final_alpha > 0.01 {
                set_pixel(canvas.frame, canvas.width, x as usize, y as usize, 
                        config::HIGHLIGHT_COLOR.0, config::HIGHLIGHT_COLOR.1, config::HIGHLIGHT_COLOR.2, final_alpha as f32);
            }
        }
    }
}
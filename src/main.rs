mod config;

use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::process;
use std::time::{Instant};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use rusttype::{Font, Scale};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::io::{self, BufRead};
use std::env;
use include_bytes; // Include the font file at compile time

static PIPE_VALUE: AtomicU32 = AtomicU32::new(u32::MAX);

struct Dial {
    cx: i32,
    cy: i32,
    r: i32,
    thickness: i32,
    arc_span: f64,
    start_angle: f64,
}

impl Dial {
    fn new(width: usize, height: usize, margin: i32, thickness: i32) -> Self {
        let cx = width as i32 / 2;
        let cy = height as i32 / 2;
        let r = (width.min(height) as i32) / 2 - margin;
        let arc_span = std::f64::consts::PI * 1.5;
        let start_angle = std::f64::consts::FRAC_PI_2;
        Self { cx, cy, r, thickness, arc_span, start_angle }
    }

    fn draw(&self, frame: &mut [u8], width: usize, height: usize, min_value: f64, max_value: f64, color: (u8, u8, u8)) {
        let end_angle = self.start_angle + self.arc_span;
        let mut start_angle = self.start_angle;
        let mut end_angle = end_angle;
        if start_angle < 0.0 { start_angle += 2.0 * std::f64::consts::PI; }
        if end_angle >= 2.0 * std::f64::consts::PI { end_angle -= 2.0 * std::f64::consts::PI; }
        // Draw arc
        for y in 0..height as i32 {
            for x in 0..width as i32 {
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
                        set_pixel(frame, width, x as usize, y as usize, color.0, color.1, color.2, aa as f32); // Use `color` for arc blending
                    }
                }
            }
        }
        // Draw tick marks and numbers
        let num_ticks = config::NUM_TICKS;
        let minor_ticks_per_interval = config::MINOR_TICKS_PER_INTERVAL;
        let tick_length = config::TICK_LENGTH;
        let minor_tick_length = config::MINOR_TICK_LENGTH;
        let tick_thickness = config::TICK_THICKNESS;
        let minor_tick_thickness = config::MINOR_TICK_THICKNESS;

        // Load font using the included font data from config.rs
        let font = Font::try_from_vec(config::FONT_DATA.to_vec()).expect("Error loading font");
        let dial_numbers_scale = Scale::uniform(config::DIAL_NUMBERS_FONT_SIZE); // Use the configured size for dial numbers
        for i in 0..num_ticks {
            let t = i as f64 / (num_ticks as f64 - 1.0);
            let angle = self.start_angle + self.arc_span * t;
            // Tick marks
            let outer_x = self.cx as f64 + angle.cos() * (self.r as f64 - 1.0);
            let outer_y = self.cy as f64 + angle.sin() * (self.r as f64 - 1.0);
            let inner_x = self.cx as f64 + angle.cos() * (self.r as f64 - tick_length as f64);
            let inner_y = self.cy as f64 + angle.sin() * (self.r as f64 - tick_length as f64);
            draw_thick_line_aa(
                frame,
                width,
                inner_x.round() as i32,
                inner_y.round() as i32,
                outer_x.round() as i32,
                outer_y.round() as i32,
                tick_thickness,
                color.0, color.1, color.2 // Set tick marks to red
            );
            // Draw minor ticks
            if i < num_ticks - 1 {
                for j in 1..=minor_ticks_per_interval {
                    let minor_t = t + (j as f64 / (minor_ticks_per_interval as f64 * (num_ticks as f64 - 1.0)));
                    let minor_angle = self.start_angle + self.arc_span * minor_t;
                    let minor_outer_x = self.cx as f64 + minor_angle.cos() * (self.r as f64 - 1.0);
                    let minor_outer_y = self.cy as f64 + minor_angle.sin() * (self.r as f64 - 1.0);
                    let minor_inner_x = self.cx as f64 + minor_angle.cos() * (self.r as f64 - minor_tick_length as f64);
                    let minor_inner_y = self.cy as f64 + minor_angle.sin() * (self.r as f64 - minor_tick_length as f64);
                    draw_thick_line_aa(
                        frame,
                        width,
                        minor_inner_x.round() as i32,
                        minor_inner_y.round() as i32,
                        minor_outer_x.round() as i32,
                        minor_outer_y.round() as i32,
                        minor_tick_thickness,
                        color.0, color.1, color.2 // Set minor tick marks to red
                    );
                }
            }
            // Draw numbers using external font, reflecting the range
            let ticks_to_numbers_distance = config::TICKS_TO_NUMBERS_DISTANCE; // Use the configured distance between ticks and numbers

            // Adjust the label radius to include the ticks-to-numbers distance
            let label_radius = self.r as f64 - tick_length as f64 - ticks_to_numbers_distance;
            let label_x = self.cx as f64 + angle.cos() * label_radius;
            let label_y = self.cy as f64 + angle.sin() * label_radius;
            let value = min_value + t * (max_value - min_value);
            // Always show as integer, no decimals
            let value_str = format!("{}", value.round() as i64);
            draw_text(frame, width, height, label_x as i32, label_y as i32, &value_str, &font, dial_numbers_scale, color);
        }
    }
}

struct Needle {
    pos: f64, // Normalized [0,1]
    phase: f64,
}

impl Needle {
    fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            pos: 0.5,
            phase: rng.random_range(0.0..1000.0),
        }
    }

    fn update(&mut self) {
        let mut rng = rand::rng();
        self.phase += rng.random_range(0.0..1000.0);
        // Instead of a pure random walk, lerp towards a new random target for smoothness
        static mut TARGET: f64 = 0.5;
        if rng.random_range(0.0..1.0) < 0.01 {
            unsafe { TARGET = rng.random_range(0.0..1.0); }
        }
        let target = unsafe { TARGET };
        self.pos = lerp(self.pos, target).clamp(0.0, 1.0);
    }

    fn draw(&self, frame: &mut [u8], width: usize, dial: &Dial, is_out_of_range: bool) {
        let angle = dial.start_angle + dial.arc_span * self.pos;
        let needle_length = dial.r as f64 * config::NEEDLE_LENGTH_FACTOR;
        let nx = (dial.cx as f64 + angle.cos() * needle_length) as i32;
        let ny = (dial.cy as f64 + angle.sin() * needle_length) as i32;

        let needle_color = if is_out_of_range { (0xff, 0x00, 0x00) } else { (0x00, 0x00, 0x00) };
        draw_thick_line_tapered_aa(frame, width, dial.cx, dial.cy, nx, ny, config::NEEDLE_WIDTH, needle_color.0, needle_color.1, needle_color.2);

        // Draw the needle's back extension
        let back_length = config::NEEDLE_BACK_LENGTH;
        let back_x = (dial.cx as f64 - angle.cos() * back_length) as i32;
        let back_y = (dial.cy as f64 - angle.sin() * back_length) as i32;
        draw_thick_line_aa(frame, width, dial.cx, dial.cy, back_x, back_y, config::NEEDLE_WIDTH, needle_color.0, needle_color.1, needle_color.2);

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
                draw_thick_line_aa(frame, width, crossbar_x1, crossbar_y1, crossbar_x2, crossbar_y2, crossbar_thickness, needle_color.0, needle_color.1, needle_color.2);
            }
            config::CrossbarType::DOT => {
                let dot_radius = config::DOT_RADIUS as i32;
                draw_circle(frame, width, dial.cx, dial.cy, dot_radius, needle_color.0, needle_color.1, needle_color.2);
            }
        }
    }
}

thread_local! {
    static NEEDLE: std::cell::RefCell<Needle> = std::cell::RefCell::new(Needle::new());
}

fn main() {
    // Parse --range x y from command line
    let mut min_value = 0.0;
    let mut max_value = 100.0;
    let mut args = env::args().peekable();
    let mut window_title = "Instrument".to_string(); // Default window title
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
        }
    }
    let logical_width: usize = config::WINDOW_WIDTH; // Use the configured window width
    let logical_height: usize = config::WINDOW_HEIGHT; // Use the configured window height
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title(&window_title) // Use the specified or default window title
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
    let start_time = Instant::now();
    let target_fps = 60.0;
    let frame_duration = std::time::Duration::from_secs_f64(1.0 / target_fps);
    let mut last_frame = Instant::now();

    // Spawn a thread to read values from stdin and update PIPE_VALUE
    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                if let Ok(val) = line.trim().parse::<f64>() {
                    let scaled = (val * 1000.0).round() as u32; // Remove clamping here
                    PIPE_VALUE.store(scaled, Ordering::Relaxed);
                }
            }
        }
    });

    event_loop.run(move |event, _window_target| {
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
                    // scale_factor = new_scale; // Commented out as it is unused
                    let size = ((400.0 * new_scale) as u32, (400.0 * new_scale) as u32);
                    fb_width = size.0 as usize;
                    fb_height = size.1 as usize;
                    let _ = inner_size_writer.request_inner_size(PhysicalSize::new(size.0, size.1));
                    let _ = pixels.resize_surface(size.0, size.1);
                }
                WindowEvent::RedrawRequested => {
                    let frame = pixels.frame_mut();
                    let elapsed = start_time.elapsed().as_secs_f64();
                    draw_dial(frame, fb_width, fb_height, elapsed, min_value, max_value);
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

fn draw_dial(frame: &mut [u8], width: usize, height: usize, _elapsed: f64, min_value: f64, max_value: f64) {
    for chunk in frame.chunks_exact_mut(4) {
        chunk.copy_from_slice(&[0xff, 0xff, 0xff, 0xff]); // Set background to white
    }
    let margin = config::DIAL_MARGIN;
    let thickness = config::DIAL_THICKNESS;
    let dial = Dial::new(width, height, margin, thickness);

    // Read PIPE_VALUE once per frame
    let piped_value = PIPE_VALUE.load(Ordering::Relaxed);
    // Determine if the piped value is out of range
    let is_out_of_range = piped_value != u32::MAX && (piped_value as f64 / 1000.0 < min_value || piped_value as f64 / 1000.0 > max_value);

    // Draw the dial based on the piped value
    let dial_color = if is_out_of_range { (0xff, 0x00, 0x00) } else { (0x00, 0x00, 0x00) }; // Red if out of range
    dial.draw(frame, width, height, min_value, max_value, dial_color);

    // Update the needle position based on the piped value
    NEEDLE.with(|needle| {
        let mut needle = needle.borrow_mut();
        if piped_value != u32::MAX {
            let v = (piped_value as f64) / 1000.0;
            let target_pos = ((v - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
            needle.pos = lerp(needle.pos, target_pos).clamp(0.0, 1.0); // Smoothly interpolate to the target position
        } else {
            needle.update();
        }
        // Needle color does not depend on out-of-range status
        needle.draw(frame, width, &dial, is_out_of_range);
    });

    // Draw the actual value in the bottom right quadrant
    let piped_value_original = PIPE_VALUE.load(Ordering::Relaxed) as f64 / 1000.0; // Use the original value
    let value_int = piped_value_original.trunc() as i32;
    let value_frac = ((piped_value_original.fract() * 1000.0).round() as u32).min(999);
    let label_x = (width as f64 * config::READOUT_X_FACTOR) as i32;
    let label_y = (height as f64 * config::READOUT_Y_FACTOR) as i32;

    // Load font once (reuse from above if possible)
    let font_data = include_bytes!("BerkeleyMono-Regular.otf");
    let font = Font::try_from_vec(font_data.to_vec()).expect("Error loading font");

    // Draw integer part in big font
    let scale_big = Scale::uniform(config::READOUT_BIG_FONT_SIZE);
    let value_str = format!("{}", value_int);

    let text_color = if is_out_of_range { (0xff, 0x00, 0x00) } else { (0x00, 0x00, 0x00) }; // Red if out of range
    draw_text(frame, width, height, label_x, label_y, &value_str, &font, scale_big, text_color);

    // Draw fractional part in smaller font, shifted further right and up for better alignment
    let int_str = format!("{}", value_int);
    let int_width = {
        use rusttype::{point, PositionedGlyph};
        let glyphs: Vec<PositionedGlyph> = font.layout(&int_str, scale_big, point(0.0, 0.0)).collect();
        let (min_x, max_x, _, _) = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).fold(
            (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
            |(min_x, max_x, min_y, max_y), bb| {
                (min_x.min(bb.min.x), max_x.max(bb.max.x), min_y, max_y)
            },
        );
        if min_x < max_x { max_x - min_x } else { 0 }
    };
    let scale_small = Scale::uniform(config::READOUT_SMALL_FONT_SIZE);
    let frac_str = format!("{:03}", value_frac);
    // Shift the fractional part further right and up
    let frac_x = label_x + int_width / 2 + 28;
    let frac_y = label_y + 2;
    draw_text(frame, width, height, frac_x, frac_y, &frac_str, &font, scale_small, text_color);

    // Add a red exclamation mark in the middle of the dial when out of range
    if is_out_of_range {
        let exclamation_x = dial.cx;
        let exclamation_y = dial.cy - (dial.r / 4); // Position just above the center
        let font_data = font_data; // Reuse the font data loaded above
        let font = Font::try_from_vec(font_data.to_vec()).expect("Error loading font");
        let scale = Scale::uniform(config::EXCLAMATION_MARK_FONT_SIZE); // Use a configured font size for the exclamation mark
        let exclamation_str = "!";
        let exclamation_color = (0xff, 0x00, 0x00); // Red color for the exclamation mark
        draw_text(frame, width, height, exclamation_x, exclamation_y, exclamation_str, &font, scale, exclamation_color);
    }
}

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
    const LERP_FACTOR: f64 = 0.1; // Adjust this factor for smoother or faster transitions
    current + (target - current) * LERP_FACTOR
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
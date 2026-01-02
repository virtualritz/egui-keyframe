//! Time ruler widget for timeline displays.

use crate::{SpaceTransform, TimeTick};
use egui::{Color32, Painter, Pos2, Rect, Stroke};

/// Configuration for the time ruler.
#[derive(Debug, Clone)]
pub struct TimeRulerConfig {
    /// Height of the ruler in pixels.
    pub height: f32,
    /// Major tick height.
    pub major_tick_height: f32,
    /// Minor tick height.
    pub minor_tick_height: f32,
    /// Text color.
    pub text_color: Color32,
    /// Tick color.
    pub tick_color: Color32,
    /// Background color.
    pub background: Color32,
}

impl Default for TimeRulerConfig {
    fn default() -> Self {
        Self {
            height: 24.0,
            major_tick_height: 12.0,
            minor_tick_height: 6.0,
            text_color: Color32::from_gray(180),
            tick_color: Color32::from_gray(100),
            background: Color32::from_gray(30),
        }
    }
}

/// Time ruler widget.
pub struct TimeRuler<'a> {
    space: &'a SpaceTransform,
    config: TimeRulerConfig,
    fps: Option<f32>,
}

impl<'a> TimeRuler<'a> {
    /// Create a new time ruler.
    pub fn new(space: &'a SpaceTransform) -> Self {
        Self {
            space,
            config: TimeRulerConfig::default(),
            fps: None,
        }
    }

    /// Set the configuration.
    pub fn config(mut self, config: TimeRulerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set FPS for frame-based display.
    pub fn fps(mut self, fps: f32) -> Self {
        self.fps = Some(fps);
        self
    }

    /// Paint the time ruler.
    pub fn paint(&self, painter: &Painter, rect: Rect) {
        // Background
        painter.rect_filled(rect, 0.0, self.config.background);

        // Determine tick spacing based on zoom
        let (major_interval, minor_count) = self.calculate_intervals();

        let (start, end) = self.space.visible_range();
        let start_val = start.value();
        let end_val = end.value();
        let first_major = (start_val / major_interval).floor() * major_interval;

        // Draw minor ticks
        let minor_interval = major_interval / minor_count as f64;
        let mut t = first_major;
        while t <= end_val + major_interval {
            for i in 0..minor_count {
                let minor_t = t + i as f64 * minor_interval;
                if minor_t >= start_val && minor_t <= end_val {
                    let x = self.space.unit_to_clipped(TimeTick::new(minor_t));
                    let is_major = i == 0;

                    let tick_height = if is_major {
                        self.config.major_tick_height
                    } else {
                        self.config.minor_tick_height
                    };

                    painter.line_segment(
                        [
                            Pos2::new(x, rect.bottom() - tick_height),
                            Pos2::new(x, rect.bottom()),
                        ],
                        Stroke::new(1.0, self.config.tick_color),
                    );

                    // Draw label for major ticks
                    if is_major {
                        let label = self.format_time(minor_t);
                        painter.text(
                            Pos2::new(x + 3.0, rect.top() + 4.0),
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::proportional(10.0),
                            self.config.text_color,
                        );
                    }
                }
            }
            t += major_interval;
        }
    }

    /// Calculate tick intervals based on zoom level.
    fn calculate_intervals(&self) -> (f64, usize) {
        let ppu = self.space.pixels_per_unit;

        // Target ~80-150 pixels between major ticks
        let target_pixels = 100.0;
        let ideal_interval = target_pixels / ppu;

        // Snap to nice intervals
        let nice_intervals = [
            0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0,
            30.0, 60.0, 120.0, 300.0, 600.0,
        ];

        let mut major_interval = 1.0;
        for &interval in &nice_intervals {
            if interval >= ideal_interval {
                major_interval = interval;
                break;
            }
        }

        // Minor tick count
        let minor_count = if major_interval >= 1.0 {
            if major_interval == 1.0 || major_interval == 2.0 {
                4
            } else if major_interval == 5.0 || major_interval == 10.0 {
                5
            } else {
                4
            }
        } else {
            5
        };

        (major_interval, minor_count)
    }

    /// Format time for display.
    fn format_time(&self, time: f64) -> String {
        if let Some(fps) = self.fps {
            // Frame-based
            let total_frames = (time * fps as f64).round() as i64;
            let seconds = total_frames / fps as i64;
            let frames = total_frames % fps as i64;

            if seconds == 0 {
                format!("{}f", frames)
            } else {
                format!("{}:{:02}f", seconds, frames.abs())
            }
        } else {
            // Time-based
            if time.abs() < 0.001 {
                "0".to_string()
            } else if time.abs() < 1.0 {
                format!("{:.0}ms", time * 1000.0)
            } else if time.abs() < 60.0 {
                if time.fract().abs() < 0.001 {
                    format!("{}s", time as i64)
                } else {
                    format!("{:.1}s", time)
                }
            } else {
                let mins = (time / 60.0).floor() as i64;
                let secs = time % 60.0;
                format!("{}:{:04.1}", mins, secs)
            }
        }
    }
}

/// Draw vertical grid lines in the track area.
pub fn draw_time_grid(
    painter: &Painter,
    rect: Rect,
    space: &SpaceTransform,
    color: Color32,
    fps: Option<f32>,
) {
    let ppu = space.pixels_per_unit;
    let target_pixels = 100.0;
    let ideal_interval = target_pixels / ppu;

    let nice_intervals = [
        0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0,
        30.0, 60.0, 120.0, 300.0, 600.0,
    ];

    let mut major_interval = 1.0;
    for &interval in &nice_intervals {
        if interval >= ideal_interval {
            major_interval = interval;
            break;
        }
    }

    let (start, end) = space.visible_range();
    let start_val = start.value();
    let end_val = end.value();
    let first = (start_val / major_interval).floor() * major_interval;

    let mut t = first;
    while t <= end_val + major_interval {
        if t >= start_val {
            let x = space.unit_to_clipped(TimeTick::new(t));
            painter.line_segment(
                [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                Stroke::new(1.0, color),
            );
        }
        t += major_interval;
    }

    // If FPS is set, draw frame lines when zoomed in enough
    if let Some(fps) = fps {
        let frame_interval = 1.0 / fps as f64;
        if space.unit_to_scaled(TimeTick::new(frame_interval)) > 10.0 {
            // At least 10 pixels per frame
            let frame_color = color.linear_multiply(0.3);
            let mut t = first;
            while t <= end_val + major_interval {
                let mut ft = t;
                while ft < t + major_interval && ft <= end_val {
                    if ft >= start_val {
                        let x = space.unit_to_clipped(TimeTick::new(ft));
                        painter.line_segment(
                            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                            Stroke::new(1.0, frame_color),
                        );
                    }
                    ft += frame_interval;
                }
                t += major_interval;
            }
        }
    }
}

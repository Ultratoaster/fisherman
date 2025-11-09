use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FishingState {
    Idle,
    Charging { power: f32 },
    Casting { 
        start_x: u16, 
        start_y: u16, 
        target_x: u16, 
        progress: f32,
    },
    Landed { 
        landing_x: u16,
        landing_y: u16,
        depth: u16,
    },
}

pub struct FishingLine {
    pub rod_x: u16,
    pub rod_y: u16,
    pub state: FishingState,
    pub color: Color,
}

impl Default for FishingLine {
    fn default() -> Self {
        Self {
            rod_x: 0,
            rod_y: 0,
            state: FishingState::Idle,
            color: Color::Rgb(200, 200, 120),
        }
    }
}

impl FishingLine {
    pub fn new(rod_x: u16, rod_y: u16) -> Self {
        Self {
            rod_x,
            rod_y,
            state: FishingState::Idle,
            ..Default::default()
        }
    }

    pub fn with_state(mut self, state: FishingState) -> Self {
        self.state = state;
        self
    }
}

fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

fn bezier_point(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let t2 = 1.0 - t;
    let x = t2 * t2 * p0.0 + 2.0 * t2 * t * p1.0 + t * t * p2.0;
    let y = t2 * t2 * p0.1 + 2.0 * t2 * t * p1.1 + t * t * p2.1;
    (x, y)
}

impl Widget for FishingLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let style = Style::default().fg(self.color);
        let hook_style = Style::default().fg(Color::Rgb(150, 150, 255));

        match self.state {
            FishingState::Idle => {
                let end_y = self.rod_y.saturating_add(3).min(area.y + area.height - 1);
                for y in self.rod_y..=end_y {
                    if self.rod_x >= area.x && self.rod_x < area.x + area.width 
                        && y >= area.y && y < area.y + area.height {
                        if y == end_y {
                            buf.set_string(self.rod_x, y, "⌡", hook_style);
                        } else {
                            buf.set_string(self.rod_x, y, "│", style);
                        }
                    }
                }
            }
            FishingState::Charging { power } => {
                let end_y = self.rod_y.saturating_add(3).min(area.y + area.height - 1);
                for y in self.rod_y..=end_y {
                    if self.rod_x >= area.x && self.rod_x < area.x + area.width 
                        && y >= area.y && y < area.y + area.height {
                        if y == end_y {
                            buf.set_string(self.rod_x, y, "⌡", hook_style);
                        } else {
                            buf.set_string(self.rod_x, y, "│", style);
                        }
                    }
                }

                let meter_y = self.rod_y.saturating_add(1);
                let meter_start_x = self.rod_x.saturating_add(2);
                let meter_length = 10;
                let filled = (power * meter_length as f32) as usize;
                
                if meter_y >= area.y && meter_y < area.y + area.height {
                    buf.set_string(meter_start_x, meter_y, "[", style);
                    for i in 0..meter_length {
                        let x = meter_start_x + 1 + i as u16;
                        if x < area.x + area.width {
                            if i < filled {
                                buf.set_string(x, meter_y, "█", Style::default().fg(Color::Green));
                            } else {
                                buf.set_string(x, meter_y, "·", Style::default().fg(Color::DarkGray));
                            }
                        }
                    }
                    let end_x = meter_start_x + 1 + meter_length as u16;
                    if end_x < area.x + area.width {
                        buf.set_string(end_x, meter_y, "]", style);
                    }
                }
            }
            FishingState::Casting { start_x: _, start_y, target_x, progress } => {
                let p0 = (self.rod_x as f32, self.rod_y as f32);
                let p2 = (target_x as f32, start_y as f32);
                
                let mid_x = (self.rod_x as f32 + target_x as f32) / 2.0;
                let horizontal_distance = (self.rod_x as f32 - target_x as f32).abs();
                let arc_height = (horizontal_distance * 0.3).min(15.0).max(5.0);
                let p1 = (mid_x, self.rod_y as f32 - arc_height);

                let current_pos = bezier_point(p0, p1, p2, progress);
                
                let points = bresenham_line(
                    self.rod_x as i32,
                    self.rod_y as i32,
                    current_pos.0 as i32,
                    current_pos.1 as i32,
                );

                for (i, (x, y)) in points.iter().enumerate() {
                    let x = *x as u16;
                    let y = *y as u16;
                    if x >= area.x && x < area.x + area.width 
                        && y >= area.y && y < area.y + area.height {
                        if i == points.len() - 1 {
                            buf.set_string(x, y, "⌡", hook_style);
                        } else {
                            buf.set_string(x, y, "·", style);
                        }
                    }
                }
            }
            FishingState::Landed { landing_x, landing_y, depth } => {
                let points_to_landing = bresenham_line(
                    self.rod_x as i32,
                    self.rod_y as i32,
                    landing_x as i32,
                    landing_y as i32,
                );

                for (i, (x, y)) in points_to_landing.iter().enumerate() {
                    let x = *x as u16;
                    let y = *y as u16;
                    if x >= area.x && x < area.x + area.width 
                        && y >= area.y && y < area.y + area.height {
                        let char = if points_to_landing.len() > 1 && i < points_to_landing.len() - 1 {
                            let (nx, ny) = points_to_landing[i + 1];
                            let dx = nx - (x as i32);
                            let dy = ny - (y as i32);
                            if dx > 0 && dy > 0 { "╲" }
                            else if dx < 0 && dy > 0 { "╱" }
                            else if dx > 0 && dy < 0 { "╱" }
                            else if dx < 0 && dy < 0 { "╲" }
                            else if dy != 0 { "│" }
                            else { "─" }
                        } else {
                            "│"
                        };
                        buf.set_string(x, y, char, style);
                    }
                }

                let vertical_start = landing_y.saturating_add(1);
                let hook_y = landing_y.saturating_add(depth);
                for y in vertical_start..=hook_y {
                    if landing_x >= area.x && landing_x < area.x + area.width 
                        && y >= area.y && y < area.y + area.height {
                        if y == hook_y {
                            buf.set_string(landing_x, y, "⌡", hook_style);
                        } else {
                            buf.set_string(landing_x, y, "│", style);
                        }
                    }
                }
            }
        }
    }
}

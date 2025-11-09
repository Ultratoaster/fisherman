use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    pub cycle_offset: f32,
}

#[derive(Clone)]
pub struct Stars {
    stars: Vec<Star>,
    elapsed: Duration,
}

impl Stars {
    pub fn new<R: Rng + ?Sized>(rng: &mut R, area: Rect, density: f32) -> Self {
        let star_count = ((area.width as f32 * area.height as f32) * density) as usize;
        let mut stars = Vec::with_capacity(star_count);
        
        for _ in 0..star_count {
            stars.push(Star {
                x: rng.gen_range(0..area.width),
                y: rng.gen_range(0..area.height),
                cycle_offset: rng.gen_range(0.0..1.0),
            });
        }
        
        Stars {
            stars,
            elapsed: Duration::ZERO,
        }
    }
    
    pub fn update(&mut self, elapsed: Duration) {
        self.elapsed = elapsed;
    }
    
    fn get_star_char(cycle_offset: f32, elapsed_secs: f32) -> &'static str {
        let cycle_duration = 3.0;
        let phase = ((elapsed_secs + cycle_offset * cycle_duration) % cycle_duration) / cycle_duration;
        
        if phase < 0.33 {
            "☼"
        } else if phase < 0.66 {
            "*"
        } else {
            "+"
        }
    }
}

impl Widget for Stars {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let elapsed_secs = self.elapsed.as_secs_f32();
        let style = Style::default().fg(Color::Rgb(200, 200, 255));
        
        for star in &self.stars {
            let x = area.x + star.x;
            let y = area.y + star.y;
            
            if x < area.x + area.width && y < area.y + area.height {
                let char = Self::get_star_char(star.cycle_offset, elapsed_secs);
                buf.set_string(x, y, char, style);
            }
        }
    }
}

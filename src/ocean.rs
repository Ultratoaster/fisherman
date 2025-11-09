use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::sync::OnceLock;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Widget;

static FOAM_SEED: OnceLock<u64> = OnceLock::new();

fn foam_seed() -> u64 {
    *FOAM_SEED.get_or_init(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    })
}

#[derive(Clone, Copy)]
pub struct Ocean;

impl Widget for Ocean {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        let surface_y = area.y;
        let fg_wave1 = Color::Rgb(102, 178, 255);
        let fg_wave2 = Color::Rgb(51, 120, 200);
        let bg_ocean = Color::Rgb(51, 51, 51);

        let mut x_off: usize = 0;
        while x_off < width {
            let x = area.x + x_off as u16;
            let pat = if (x_off % 7) == 0 { "~~" } else if (x_off % 5) == 0 { "~~" } else { "~" };
            let fg = if x_off % 2 == 0 { fg_wave1 } else { fg_wave2 };
            buf.set_string(x, surface_y, pat, Style::default().fg(fg).bg(bg_ocean));
            x_off += pat.chars().count();
        }

        for foam_row in 1..=3u16 {
            let y = area.y + foam_row;
            if y >= area.y + area.height { break; }

            let mut x_off: u16 = 0;
            let base_seed = foam_seed();
            let seed = base_seed
                ^ ((area.x as u64) << 48)
                ^ ((area.y as u64) << 32)
                ^ ((foam_row as u64) << 16)
                ^ (area.width as u64);
            let mut rng = StdRng::seed_from_u64(seed);
            while x_off < area.width {
                    if rng.gen_bool(0.18) {
                    let u1 = rng.gen_range(0.0f32..1.0f32);
                    let u2 = rng.gen_range(0.0f32..1.0f32);
                    let t = (u1 + u2) / 2.0;
                    let mut len = (t * 6.0).floor() as u16 + 2; // 2..=7
                    if len < 2 { len = 2; }
                    if len > 7 { len = 7; }

                    for i in 0..len {
                        if x_off + i >= area.width { break; }
                        let x = area.x + (x_off + i);
                        buf.set_string(x, y, "^", Style::default().fg(Color::Rgb(200,220,255)).bg(bg_ocean));
                    }
                    x_off = x_off.saturating_add(len);
                } else {
                    x_off = x_off.saturating_add(1);
                }
            }
        }
    }
}

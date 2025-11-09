use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Widget;

pub struct FishermanDock {
    pub width: u16,
}

impl Widget for FishermanDock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let plank = "═";
        let plank_post = "╦";
        let post = "║";
        let end_plank = "╔";
        let plank_color = Color::Rgb(101, 67, 33);
        let post_color = Color::Rgb(80, 50, 20);

        let total_height = area.height.min(4);
        let y = area.y + area.height.saturating_sub(total_height);
        let dock_w = self.width.min(area.width) as usize;

        let right_x = area.x.saturating_add(area.width.saturating_sub(1));

        let mut has_post = vec![false; dock_w];
        if dock_w > 0 {
            let mut left_off = 0usize;
            while left_off < dock_w {
                let idx = dock_w - 1 - left_off;
                has_post[idx] = true;
                left_off = left_off.saturating_add(2);
            }
        }

        for x_off in 0..dock_w {
            let x = right_x.saturating_sub(x_off as u16);
            if x_off == dock_w - 1 {
                buf.set_string(x, y, end_plank, Style::default().fg(plank_color));
            } else if has_post[x_off] {
                buf.set_string(x, y, plank_post, Style::default().fg(plank_color));
            } else {
                buf.set_string(x, y, plank, Style::default().fg(plank_color));
            }
        }

        let post_h: u16 = 2;
        for x_off in 0..dock_w {
            if !has_post[x_off] { continue; }
            let x = right_x.saturating_sub(x_off as u16);
            for r in 1..=post_h {
                let yy = y + r;
                if yy < area.y + area.height {
                    buf.set_string(x, yy, post, Style::default().fg(post_color));
                }
            }
        }
    }
}

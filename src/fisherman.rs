use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Widget;

/// A small widget that draws a fisherman stick figure.
pub struct Fisherman {
    pub offset_from_right: u16,
    pub kick: bool,
}

impl Widget for Fisherman {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let right_x = area.x.saturating_add(area.width.saturating_sub(1));
        let fx = right_x.saturating_sub(self.offset_from_right.min(area.width.saturating_sub(1)));
        let head_y = area.y;

        buf.set_string(
            fx,
            head_y,
            "ö",
            Style::default().fg(Color::Rgb(200, 200, 200)),
        );
        if head_y + 1 < area.y + area.height {
            buf.set_string(
                fx,
                head_y + 1,
                "┤",
                Style::default().fg(Color::Rgb(200, 200, 200)),
            );
        }
        if head_y + 2 < area.y + area.height {
            if fx > area.x {
                buf.set_string(
                    fx,
                    head_y + 2,
                    "┘",
                    Style::default().fg(Color::Rgb(200, 200, 200)),
                );
                if self.kick {
                    buf.set_string(
                        fx - 1,
                        head_y + 2,
                        "─",
                        Style::default().fg(Color::Rgb(200, 200, 200)),
                    );
                } else {
                    buf.set_string(
                        fx - 1,
                        head_y + 2,
                        "┌",
                        Style::default().fg(Color::Rgb(200, 200, 200)),
                    );
                }
            }
        }

        let rod_length = 4;
        for i in 0..rod_length {
            if fx > area.x + 1 && head_y >= area.y {
                buf.set_string(
                    fx - (i + 1),
                    head_y - i,
                    "\\",
                    Style::default().fg(Color::Rgb(200, 200, 120)),
                );
            }
        }
    }
}

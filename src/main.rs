use std::io;
use std::time::{Duration, Instant};
use std::thread;

mod csv_frames;
mod ocean;
mod widgets;
mod fisherman;
mod fish;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui::text::Text;
use ratatui::layout::Rect;
use rand;

use fish::{Fish, spawn_fishes};
use ocean::Ocean;
use widgets::FishermanDock;
use fisherman::Fisherman;
use csv_frames::load_frames_from_dir;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut per_species = match csv_frames::load_all_fish_frames("src/fish") {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };
    if per_species.is_empty() {
        let fallback = load_frames_from_dir("src/fish").unwrap_or_else(|_| Vec::new());
        let fr = load_frames_from_dir("src/fish/right").unwrap_or_else(|_| fallback.clone());
        let fl = load_frames_from_dir("src/fish/left").unwrap_or_else(|_| Vec::new());
        per_species.push((fr, fl));
    }

    let mut rng = rand::thread_rng();

    let has_left = per_species.iter().any(|(_, l)| !l.is_empty());
    let has_right = per_species.iter().any(|(r, _)| !r.is_empty());
    let species_count = per_species.len();

    let screen_width = match terminal.size() {
        Ok(s) => s.width as f32,
        Err(_) => 80.0,
    };
    let mut fishes: Vec<Fish> = spawn_fishes(&mut rng, has_left, has_right, species_count, screen_width);

    let start = Instant::now();
    let load_time = Duration::from_secs(30);

    let mut last_update = Instant::now();
    let mut fisherman_kick = false;
    let mut last_kick_toggle = Instant::now();
    let kick_interval = Duration::from_millis(400);
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_update);
        last_update = now;
        let elapsed = start.elapsed();

        if now.duration_since(last_kick_toggle) >= kick_interval {
            fisherman_kick = !fisherman_kick;
            last_kick_toggle = now;
        }

        if !fishes.is_empty() {
            if let Ok(size) = terminal.size() {
                let width = size.width as f32;
                for fish in fishes.iter_mut() {
                    if elapsed.as_millis() < fish.spawn_delay_ms as u128 {
                        continue;
                    }
                    fish.x += fish.vx * dt.as_secs_f32();
                    if fish.x > width {
                        if fish.wrap {
                            fish.x = 0.0;
                        } else {
                            fish.x = width.min(fish.x);
                            if has_left && has_right {
                                fish.vx = -fish.vx;
                                fish.facing_right = !fish.facing_right;
                            } else {
                                fish.vx = 0.0;
                            }
                        }
                    } else if fish.x < 0.0 {
                        if fish.wrap {
                            fish.x = width;
                        } else {
                            fish.x = 0.0;
                            if has_left && has_right {
                                fish.vx = -fish.vx;
                                fish.facing_right = !fish.facing_right;
                            } else {
                                fish.vx = 0.0;
                            }
                        }
                    }
                }
            }
        }

        terminal.draw(|f| {
            let size = f.area();
            let ocean_h = 4u16;
            let desired_top: u16 = 20; //distance from top of terminal
            let top = if size.height > desired_top + ocean_h {
                desired_top
            } else if size.height > ocean_h {
                size.height.saturating_sub(ocean_h)
            } else {
                0
            };
            let ocean_area = Rect::new(size.x+1, top, size.width-2, ocean_h);
            f.render_widget(Ocean, ocean_area);
            // Render dock
            let dock_w = 16u16;
            let dock_h = 4u16;
                let dock_x = size.x.saturating_add(size.width.saturating_sub(dock_w));
                let dock_y = ocean_area.y.saturating_sub(1) - 1;
                let dock_area = Rect::new(dock_x-1, dock_y, dock_w, dock_h);
            f.render_widget(FishermanDock { width: dock_w }, dock_area);
                // Render fisherman
                let fisher_h = 9u16;
                let fisher_y = dock_area.y-2; // dock_area.y was chosen as one row above plank
                let fisher_area = Rect::new(dock_x-(dock_w-1), fisher_y, dock_w, fisher_h);
                let fisher = Fisherman { offset_from_right: 1, kick: fisherman_kick };
                f.render_widget(fisher, fisher_area);

            // Compute fish layout and rendering operations
            let (lanes, lane_height, _base_y) = fish::compute_fish_layout(size);
            let fish_area_height = lane_height.saturating_mul(lanes) - 2;
            let desired_top = ocean_area.y.saturating_add(5);
            let base_y = if desired_top.saturating_add(fish_area_height) <= size.height {
                desired_top
            } else if size.height > fish_area_height {
                size.height.saturating_sub(fish_area_height)
            } else {
                0
            };
            let fish_group_area = Rect::new(size.x, base_y, size.width, fish_area_height);

            // Ask the fish module to compute rendering operations for fish
            let ops = fish::compute_fish_render_ops(&fishes, fish_group_area, &per_species, elapsed);
            for (rect, text) in ops.into_iter() {
                let fish_par = Paragraph::new(text).block(Block::default());
                f.render_widget(fish_par, rect);
            }

            // If loading complete, show "Got one!" message in fisherman area
            if elapsed >= load_time {
                let done_par = Paragraph::new(Text::from("Got one!")).block(
                    Block::default().title("Fisherman").borders(Borders::ALL),
                );
                f.render_widget(done_par, size);
            } else {
                let block = Block::default().title("Fisherman").borders(Borders::ALL);
                f.render_widget(block, size);
            }
        })?;

        // Exit when fish caught or user presses 'q'
        if elapsed >= load_time {
            thread::sleep(Duration::from_secs(2));
            break;
        }
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

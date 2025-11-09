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

// Layout constants
const OCEAN_HEIGHT: u16 = 4;
const OCEAN_DESIRED_TOP: u16 = 20;
const DOCK_WIDTH: u16 = 16;
const DOCK_HEIGHT: u16 = 4;
const FISHERMAN_HEIGHT: u16 = 9;
const FISH_AREA_OFFSET_FROM_OCEAN: u16 = 5;

/// Compute the ocean area placement given the terminal size
fn compute_ocean_area(size: Rect) -> Rect {
    let top = if size.height > OCEAN_DESIRED_TOP + OCEAN_HEIGHT {
        OCEAN_DESIRED_TOP
    } else if size.height > OCEAN_HEIGHT {
        size.height.saturating_sub(OCEAN_HEIGHT)
    } else {
        0
    };
    Rect::new(size.x + 1, top, size.width - 2, OCEAN_HEIGHT)
}

/// Compute fish area placement and lane count based on ocean position
fn compute_fish_area(size: Rect, ocean_y: u16) -> (Rect, u16) {
    let lane_height = fish::FISH_HEIGHT;
    let desired_top = ocean_y.saturating_add(FISH_AREA_OFFSET_FROM_OCEAN);
    let available_height = if desired_top < size.height {
        size.height.saturating_sub(desired_top)
    } else {
        0
    };
    let lanes = std::cmp::max(1u16, available_height / lane_height);
    let fish_area_height = lane_height.saturating_mul(lanes).saturating_sub(2);
    let base_y = if desired_top.saturating_add(fish_area_height) <= size.height {
        desired_top
    } else if size.height > fish_area_height {
        size.height.saturating_sub(fish_area_height)
    } else {
        0
    };
    (Rect::new(size.x, base_y, size.width, fish_area_height), lanes)
}

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

    // Get initial terminal size for spawn calculations
    let initial_size = match terminal.size() {
        Ok(s) => Rect::new(0, 0, s.width, s.height),
        Err(_) => Rect::new(0, 0, 80, 24),
    };
    let ocean_area = compute_ocean_area(initial_size);
    let (_, lanes) = compute_fish_area(initial_size, ocean_area.y);

    let mut fishes: Vec<Fish> = spawn_fishes(
        &mut rng,
        &per_species,
        initial_size.width as f32,
        lanes as usize,
    );

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
                    
                    // Handle edge wrapping/bouncing
                    let out_of_bounds = if fish.x > width {
                        Some((width, 0.0)) // (clamp_value, wrap_value)
                    } else if fish.x < 0.0 {
                        Some((0.0, width))
                    } else {
                        None
                    };
                    
                    if let Some((clamp_pos, wrap_pos)) = out_of_bounds {
                        if fish.wrap {
                            fish.x = wrap_pos;
                        } else {
                            fish.x = clamp_pos;
                            // Check if this fish can bounce (has both directions, none implemented yet.)
                            let (species_has_right, species_has_left) = 
                                fish::species_has_directions(&per_species, fish.species);
                            if species_has_left && species_has_right {
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
            
            // Render ocean
            let ocean_area = compute_ocean_area(size);
            f.render_widget(Ocean, ocean_area);
            
            // Render dock
            let dock_x = size.x.saturating_add(size.width.saturating_sub(DOCK_WIDTH));
            let dock_y = ocean_area.y.saturating_sub(2);
            let dock_area = Rect::new(dock_x - 1, dock_y, DOCK_WIDTH, DOCK_HEIGHT);
            f.render_widget(FishermanDock { width: DOCK_WIDTH }, dock_area);
            
            // Render fisherman
            let fisher_y = dock_area.y - 2;
            let fisher_area = Rect::new(dock_x - (DOCK_WIDTH - 1), fisher_y, DOCK_WIDTH, FISHERMAN_HEIGHT);
            let fisher = Fisherman { offset_from_right: 1, kick: fisherman_kick };
            f.render_widget(fisher, fisher_area);

            // Compute fish area and render fish
            let (fish_group_area, _) = compute_fish_area(size, ocean_area.y);
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

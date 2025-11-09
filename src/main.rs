use std::io;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::env;
use std::path::PathBuf;
use std::fs;

#[cfg(windows)]
use std::fs::OpenOptions;

mod csv_frames;
mod ocean;
mod widgets;
mod fisherman;
mod fish;
mod fishing_line;
mod fishing_game;
mod stars;

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
use fishing_line::{FishingLine, FishingState};
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
    let args: Vec<String> = env::args().collect();
    let subprocess_mode = args.contains(&"--subprocess".to_string());
    
    // Check for --pipe argument (named pipe path)
    let pipe_path: Option<PathBuf> = args.iter()
        .position(|arg| arg == "--pipe")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    
    // Check for --signal-file argument (backward compatibility)
    let signal_file: Option<PathBuf> = args.iter()
        .position(|arg| arg == "--signal-file")
        .and_then(|i| args.get(i + 1))
        .map(PathBuf::from);
    
    // Shared signal state
    let signal_received: Arc<Mutex<Option<(bool, String)>>> = Arc::new(Mutex::new(None));
    
    // If in subprocess mode, spawn a thread to read from stdin
    if subprocess_mode {
        let signal_clone = Arc::clone(&signal_received);
        thread::spawn(move || {
            let stdin = io::stdin();
            let reader = BufReader::new(stdin);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let line = line.trim();
                    if let Some(msg) = line.strip_prefix("SUCCESS:") {
                        *signal_clone.lock().unwrap() = Some((true, msg.to_string()));
                    } else if let Some(msg) = line.strip_prefix("FAILURE:") {
                        *signal_clone.lock().unwrap() = Some((false, msg.to_string()));
                    }
                }
            }
        });
    }
    
    // If named pipe is specified, read from it in a thread
    if let Some(ref path) = pipe_path {
        let signal_clone = Arc::clone(&signal_received);
        let path = path.clone();
        thread::spawn(move || {
            #[cfg(windows)]
            {
                // Windows named pipe: \\.\pipe\name
                loop {
                    if let Ok(file) = OpenOptions::new().read(true).open(&path) {
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                let line = line.trim();
                                if let Some(msg) = line.strip_prefix("SUCCESS:") {
                                    *signal_clone.lock().unwrap() = Some((true, msg.to_string()));
                                } else if let Some(msg) = line.strip_prefix("FAILURE:") {
                                    *signal_clone.lock().unwrap() = Some((false, msg.to_string()));
                                }
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
            #[cfg(not(windows))]
            {
                // Unix named pipe (FIFO)
                if let Ok(file) = std::fs::File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            let line = line.trim();
                            if let Some(msg) = line.strip_prefix("SUCCESS:") {
                                *signal_clone.lock().unwrap() = Some((true, msg.to_string()));
                            } else if let Some(msg) = line.strip_prefix("FAILURE:") {
                                *signal_clone.lock().unwrap() = Some((false, msg.to_string()));
                            }
                        }
                    }
                }
            }
        });
    }
    
    // If signal file is specified, poll it in a thread (backward compatibility)
    if let Some(ref path) = signal_file {
        let signal_clone = Arc::clone(&signal_received);
        let path = path.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                if let Ok(content) = fs::read_to_string(&path) {
                    let content = content.trim();
                    if !content.is_empty() {
                        if let Some(msg) = content.strip_prefix("SUCCESS:") {
                            *signal_clone.lock().unwrap() = Some((true, msg.to_string()));
                            let _ = fs::write(&path, ""); // Clear the file
                        } else if let Some(msg) = content.strip_prefix("FAILURE:") {
                            *signal_clone.lock().unwrap() = Some((false, msg.to_string()));
                            let _ = fs::write(&path, ""); // Clear the file
                        }
                    }
                }
            }
        });
    }
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let moon_sprite = csv_frames::load_moon_embedded()
        .ok()
        .or_else(|| csv_frames::load_csv_frame("moon.csv").ok());

    let species_list = match csv_frames::load_all_fish_species_embedded() {
        Ok(v) if !v.is_empty() => v,
        _ => {
            match csv_frames::load_all_fish_species("src/fish") {
                Ok(v) => v,
                Err(_) => Vec::new(),
            }
        }
    };
    let mut per_species: Vec<_> = species_list.iter().map(|s| s.frames.clone()).collect();
    if per_species.is_empty() {
        let fallback = load_frames_from_dir("src/fish").unwrap_or_else(|_| Vec::new());
        let fr = load_frames_from_dir("src/fish/right").unwrap_or_else(|_| fallback.clone());
        let fl = load_frames_from_dir("src/fish/left").unwrap_or_else(|_| Vec::new());
        per_species.push((fr, fl));
    }

    let mut rng = rand::thread_rng();

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

    let mut last_update = Instant::now();
    let mut fisherman_kick = false;
    let mut last_kick_toggle = Instant::now();
    let kick_interval = Duration::from_millis(400);
    
    let mut last_spawn_check = Instant::now();
    let spawn_check_interval = Duration::from_secs(3);
    
    let mut fishing_state = FishingState::Idle;
    let mut cast_charge_start: Option<Instant> = None;
    let max_cast_time = Duration::from_secs(2);
    let mut cast_animation_start: Option<Instant> = None;
    let cast_animation_duration = Duration::from_millis(800);
    
    let mut caught_fish: Option<fishing_game::CaughtFish> = None;
    let mut catch_message_shown_at: Option<Instant> = None;
    
    let mut local_signal: Option<(bool, String)> = None;
    
    let sky_height = ocean_area.y;
    let sky_area = Rect::new(0, 0, initial_size.width, sky_height);
    let mut stars_widget = stars::Stars::new(&mut rng, sky_area, 0.02);
    let mut last_window_size = (initial_size.width, initial_size.height);
    
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_update);
        last_update = now;
        let elapsed = start.elapsed();
        
        // Check for signals from subprocess stdin, pipe, or signal file
        if subprocess_mode || pipe_path.is_some() || signal_file.is_some() {
            if let Ok(mut sig) = signal_received.lock() {
                if sig.is_some() {
                    local_signal = sig.take();
                    fisherman_kick = local_signal.as_ref().map(|(success, _)| *success).unwrap_or(false);
                }
            }
        }

        if now.duration_since(last_kick_toggle) >= kick_interval {
            fisherman_kick = !fisherman_kick;
            last_kick_toggle = now;
        }
        
        stars_widget.update(elapsed);

        if now.duration_since(last_spawn_check) >= spawn_check_interval {
            last_spawn_check = now;
            if let Ok(size) = terminal.size() {
                let ocean_area = compute_ocean_area(Rect::new(0, 0, size.width, size.height));
                let (_, lanes) = compute_fish_area(Rect::new(0, 0, size.width, size.height), ocean_area.y);
                
                let current_fish_count = fishes.len();
                let target_fish_count = lanes as usize;
                
                if current_fish_count < target_fish_count {
                    let mut new_fish = spawn_fishes(
                        &mut rng,
                        &per_species,
                        size.width as f32,
                        lanes as usize,
                    );
                    fishes.append(&mut new_fish);
                }
            }
        }

        if let Some(anim_start) = cast_animation_start {
            let anim_elapsed = now.duration_since(anim_start);
            if anim_elapsed < cast_animation_duration {
                if let FishingState::Casting { start_x, start_y, target_x, progress: _ } = fishing_state {
                    let new_progress = anim_elapsed.as_secs_f32() / cast_animation_duration.as_secs_f32();
                    fishing_state = FishingState::Casting {
                        start_x,
                        start_y,
                        target_x,
                        progress: new_progress,
                    };
                }
            } else {
                if let FishingState::Casting { target_x, start_y, .. } = fishing_state {
                    fishing_state = FishingState::Landed {
                        landing_x: target_x,
                        landing_y: start_y,
                        depth: 0,
                    };
                }
                cast_animation_start = None;
            }
        }

        if let Some(charge_start) = cast_charge_start {
            let charge_elapsed = now.duration_since(charge_start);
            let power = (charge_elapsed.as_secs_f32() / max_cast_time.as_secs_f32()).min(1.0);
            fishing_state = FishingState::Charging { power };
        }

        if !fishes.is_empty() {
            if let Ok(size) = terminal.size() {
                let width = size.width as f32;
                for fish in fishes.iter_mut() {
                    if elapsed.as_millis() < fish.spawn_delay_ms as u128 {
                        continue;
                    }
                    fish.x += fish.vx * dt.as_secs_f32();
                    
                    let out_of_bounds = if fish.x > width {
                        Some((width, 0.0))
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
                            let (species_has_right, species_has_left) = 
                                fish::species_has_directions(&per_species, fish.species);
                            if species_has_left && species_has_right {
                                fish.vx = -fish.vx;
                                fish.facing_right = !fish.facing_right;
                            }
                        }
                    }
                }
                
                if let FishingState::Landed { landing_x, landing_y, depth } = fishing_state {
                    let hook_x = landing_x;
                    let hook_y = landing_y.saturating_add(depth);
                    let ocean_area = compute_ocean_area(Rect::new(0, 0, size.width, size.height));
                    let (fish_area, _) = compute_fish_area(Rect::new(0, 0, size.width, size.height), ocean_area.y);
                    
                    // Check each fish for collision
                    for (i, fish) in fishes.iter().enumerate() {
                        if elapsed.as_millis() < fish.spawn_delay_ms as u128 {
                            continue;
                        }
                        
                        let fish_y = fish_area.y + (fish.lane as u16 * fish::FISH_HEIGHT) + fish::FISH_HEIGHT / 2;
                        let fish_width = 22; // Approximate fish width from CSV
                        let fish_height = fish::FISH_HEIGHT;
                        
                        if fishing_game::check_collision(hook_x, hook_y, fish.x, fish_y, fish_width, fish_height) {
                            // Fish caught!
                            let species_name = if fish.species < species_list.len() {
                                species_list[fish.species].name.clone()
                            } else {
                                "Unknown Fish".to_string()
                            };
                            
                            caught_fish = Some(fishing_game::CaughtFish::new(species_name, fish.size));
                            catch_message_shown_at = Some(now);
                            
                            fishes.remove(i);
                            
                            fishing_state = FishingState::Idle;
                            break;
                        }
                    }
                }
            }
        }

        terminal.draw(|f| {
            let size = f.area();
            
            let ocean_area = compute_ocean_area(size);
            f.render_widget(Ocean, ocean_area);
            
            let sky_area = Rect::new(0, 0, size.width, ocean_area.y);
            f.render_widget(stars_widget.clone(), sky_area);
            
            if let Some(ref moon) = moon_sprite {
                let moon_x = 8;
                let moon_y = 3;
                let moon_area = Rect::new(moon_x, moon_y, 10, 7);
                let moon_par = Paragraph::new(moon.clone()).block(Block::default());
                f.render_widget(moon_par, moon_area);
            }
            
            let dock_x = size.x.saturating_add(size.width.saturating_sub(DOCK_WIDTH));
            let dock_y = ocean_area.y.saturating_sub(2);
            let dock_area = Rect::new(dock_x - 1, dock_y, DOCK_WIDTH, DOCK_HEIGHT);
            f.render_widget(FishermanDock { width: DOCK_WIDTH }, dock_area);
            
            let fisher_y = dock_area.y - 2;
            let fisher_area = Rect::new(dock_x - (DOCK_WIDTH - 1), fisher_y, DOCK_WIDTH, FISHERMAN_HEIGHT);
            let fisher = Fisherman { offset_from_right: 1, kick: fisherman_kick };
            f.render_widget(fisher, fisher_area);
            
            if local_signal.is_some() {
                let exclaim_x = dock_x - (DOCK_WIDTH / 2);
                let exclaim_y = fisher_y.saturating_sub(1);
                if exclaim_y < size.height {
                    let exclaim_style = ratatui::style::Style::default()
                        .fg(ratatui::style::Color::Yellow);
                    f.buffer_mut().set_string(exclaim_x, exclaim_y, "!", exclaim_style);
                }
            }

            let rod_tip_x = dock_x - 1 - 4 - 1;
            let rod_tip_y = fisher_y.saturating_sub(4).saturating_add(2).saturating_sub(1);
            let fishing_line = FishingLine::new(rod_tip_x, rod_tip_y).with_state(fishing_state);
            f.render_widget(fishing_line, size);

            let (fish_group_area, _) = compute_fish_area(size, ocean_area.y);
            let ops = fish::compute_fish_render_ops(&fishes, fish_group_area, &per_species, elapsed);
            for (rect, text) in ops.into_iter() {
                let fish_par = Paragraph::new(text).block(Block::default());
                f.render_widget(fish_par, rect);
            }

            if let Some(ref caught) = caught_fish {
                // Show caught fish message
                let message = caught.format_catch();
                let catch_par = Paragraph::new(Text::from(message))
                    .block(Block::default().title("Nice Catch!").borders(Borders::ALL))
                    .style(ratatui::style::Style::default().fg(ratatui::style::Color::Green));
                
                // Center the message box
                let msg_width = 40;
                let msg_height = 6;
                let msg_x = size.width.saturating_sub(msg_width) / 2;
                let msg_y = size.height.saturating_sub(msg_height) / 2;
                let msg_area = Rect::new(msg_x, msg_y, msg_width, msg_height);
                f.render_widget(catch_par, msg_area);
            } else {
                let block = Block::default().title("Fisherman").borders(Borders::ALL);
                f.render_widget(block, size);
            }
            
            if let Some((is_success, ref message)) = local_signal {
                let color = if is_success {
                    ratatui::style::Color::Green
                } else {
                    ratatui::style::Color::Red
                };
                let signal_par = Paragraph::new(Text::from(message.as_str()))
                    .block(Block::default().borders(Borders::ALL))
                    .style(ratatui::style::Style::default().fg(color))
                    .alignment(ratatui::layout::Alignment::Center);
                
                // Position in the upper part of the sky
                let msg_width = message.len().min(60) as u16 + 4;
                let msg_height = 3;
                let msg_x = size.width.saturating_sub(msg_width) / 2;
                let msg_y = ocean_area.y / 3; // Upper third of sky
                let msg_area = Rect::new(msg_x, msg_y, msg_width, msg_height);
                f.render_widget(signal_par, msg_area);
            }
        })?;

        if let Some(shown_at) = catch_message_shown_at {
            if now.duration_since(shown_at) > Duration::from_secs(3) {
                caught_fish = None;
                catch_message_shown_at = None;
            }
        }

        if local_signal.is_some() {
            thread::sleep(Duration::from_secs(3));
            break;
        }
        
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Resize(width, height) => {
                    if (width, height) != last_window_size {
                        last_window_size = (width, height);
                        let new_size = Rect::new(0, 0, width, height);
                        let ocean_area = compute_ocean_area(new_size);
                        let sky_height = ocean_area.y;
                        let sky_area = Rect::new(0, 0, width, sky_height);
                        stars_widget = stars::Stars::new(&mut rng, sky_area, 0.02);
                        stars_widget.update(elapsed);
                    }
                }
                Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        match key.kind {
                            event::KeyEventKind::Press => {
                                if matches!(fishing_state, FishingState::Idle) {
                                    cast_charge_start = Some(now);
                                } else if let FishingState::Charging { power } = fishing_state {
                                    // On Linux, key release may not fire, so allow pressing space again to cast
                                    if let Ok(size) = terminal.size() {
                                        let screen_width = size.width;
                                        let ocean_area = compute_ocean_area(Rect::new(0, 0, size.width, size.height));
                                        let rod_tip_x = screen_width.saturating_sub(DOCK_WIDTH)
                                            .saturating_sub(1)
                                            .saturating_sub(4)
                                            .saturating_sub(1);
                                        let dock_y = ocean_area.y.saturating_sub(2);
                                        let _rod_tip_y = dock_y.saturating_sub(2).saturating_sub(4).saturating_add(2).saturating_sub(1);
                                        
                                        let max_distance = (screen_width as f32 * 0.7) as u16;
                                        let cast_distance = (max_distance as f32 * power) as u16;
                                        let target_x = rod_tip_x.saturating_sub(cast_distance.max(10));
                                        let landing_y = ocean_area.y;
                                        
                                        fishing_state = FishingState::Casting {
                                            start_x: rod_tip_x,
                                            start_y: landing_y,
                                            target_x,
                                            progress: 0.0,
                                        };
                                        cast_animation_start = Some(now);
                                    }
                                    cast_charge_start = None;
                                }
                            }
                            event::KeyEventKind::Release => {
                                if let FishingState::Charging { power } = fishing_state {
                                    if let Ok(size) = terminal.size() {
                                        let screen_width = size.width;
                                        let ocean_area = compute_ocean_area(Rect::new(0, 0, size.width, size.height));
                                        let rod_tip_x = screen_width.saturating_sub(DOCK_WIDTH)
                                            .saturating_sub(1)
                                            .saturating_sub(4)
                                            .saturating_sub(1);
                                        let dock_y = ocean_area.y.saturating_sub(2);
                                        let _rod_tip_y = dock_y.saturating_sub(2).saturating_sub(4).saturating_add(2).saturating_sub(1);
                                        
                                        let max_distance = (screen_width as f32 * 0.7) as u16;
                                        let cast_distance = (max_distance as f32 * power) as u16;
                                        let target_x = rod_tip_x.saturating_sub(cast_distance.max(10));
                                        let landing_y = ocean_area.y;
                                        
                                        fishing_state = FishingState::Casting {
                                            start_x: rod_tip_x,
                                            start_y: landing_y,
                                            target_x,
                                            progress: 0.0,
                                        };
                                        cast_animation_start = Some(now);
                                    }
                                    cast_charge_start = None;
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Down => {
                        if let FishingState::Landed { landing_x, landing_y, depth } = fishing_state {
                            let max_depth = terminal.size().map(|s| s.height.saturating_sub(landing_y)).unwrap_or(30);
                            fishing_state = FishingState::Landed {
                                landing_x,
                                landing_y,
                                depth: depth.saturating_add(1).min(max_depth),
                            };
                        }
                    }
                    KeyCode::Up => {
                        if let FishingState::Landed { landing_x, landing_y, depth } = fishing_state {
                            if depth == 0 {
                                fishing_state = FishingState::Idle;
                            } else {
                                fishing_state = FishingState::Landed {
                                    landing_x,
                                    landing_y,
                                    depth: depth.saturating_sub(1),
                                };
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        // Test signal: SUCCESS (works when not using external signals)
                        if !subprocess_mode && pipe_path.is_none() && signal_file.is_none() {
                            local_signal = Some((true, "Success! Task completed.".to_string()));
                            fisherman_kick = true;
                        }
                    }
                    KeyCode::Char('f') => {
                        // Test signal: FAILURE (works when not using external signals)
                        if !subprocess_mode && pipe_path.is_none() && signal_file.is_none() {
                            local_signal = Some((false, "Failed! Please try again.".to_string()));
                            fisherman_kick = false;
                        }
                    }
                    _ => {}
                }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

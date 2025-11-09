use std::time::Duration;
use ratatui::text::Text;
use ratatui::layout::Rect;

#[derive(Debug, Clone)]
pub struct Fish {
    pub lane: usize,
    pub x: f32,
    pub vx: f32,
    pub wrap: bool,
    pub facing_right: bool,
    /// index into the per-species frames vector so each fish keeps its
    /// assigned species' sprites for the lifetime of the fish.
    pub species: usize,
    pub frame_duration: Duration,
    /// Delay (ms) before this fish appears; used to stagger spawns.
    pub spawn_delay_ms: u64,
}

// Layout constants
pub const FISH_HEIGHT: u16 = 6;
const FISH_Y_OFFSET: u16 = 2;

// Spawn tuning constants
const BASE_SPAWN_CHANCE: f64 = 0.6;
const BASE_SCREEN_WIDTH: f32 = 80.0;
const MAX_SPAWN_CHANCE: f64 = 0.95;
const MIN_WIDTH_FACTOR: f32 = 0.5;
const MAX_SPAWN_DELAY_MS: u64 = 5000;
const DEFAULT_FRAME_DURATION_MS: u64 = 150;
const EDGE_SPAWN_OFFSET: f32 = 8.0;

/// Select the appropriate frame vector for a fish based on facing direction.
/// Returns the right-facing frames if left-facing frames don't exist.
fn select_frames<'a>(
    frames_by_species: &'a [(Vec<Text<'a>>, Vec<Text<'a>>)],
    species_idx: usize,
    facing_right: bool,
) -> &'a [Text<'a>] {
    if frames_by_species.is_empty() {
        return &[];
    }
    
    let species_idx = species_idx.min(frames_by_species.len() - 1);
    let (ref_vec_right, ref_vec_left) = &frames_by_species[species_idx];
    
    if facing_right || ref_vec_left.is_empty() {
        ref_vec_right.as_slice()
    } else {
        ref_vec_left.as_slice()
    }
}

/// Calculate spawn probability based on screen width
fn compute_spawn_chance(screen_width: f32) -> f64 {
    let w_factor = (screen_width / BASE_SCREEN_WIDTH).max(MIN_WIDTH_FACTOR);
    let chance = BASE_SPAWN_CHANCE * (w_factor as f64);
    chance.min(MAX_SPAWN_CHANCE)
}

/// Calculate initial X position for spawning at screen edge
fn compute_spawn_x<R: rand::Rng + ?Sized>(rng: &mut R, dir_right: bool, screen_width: f32) -> f32 {
    if dir_right {
        // Start a bit off the left edge
        rng.gen_range(-EDGE_SPAWN_OFFSET..0.0)
    } else {
        // Start a bit off the right edge
        rng.gen_range(screen_width..(screen_width + EDGE_SPAWN_OFFSET))
    }
}

/// Check if a specific species has frames for each direction
pub fn species_has_directions(
    frames_by_species: &[(Vec<Text>, Vec<Text>)],
    species_idx: usize,
) -> (bool, bool) {
    if species_idx >= frames_by_species.len() {
        return (false, false);
    }
    let (right_frames, left_frames) = &frames_by_species[species_idx];
    (!right_frames.is_empty(), !left_frames.is_empty())
}

/// Compute layout parameters for fish lanes given the full terminal area.
/// Returns (lanes, lane_height, base_y) where `base_y` is the top row where
/// the grouped fish lanes start.
pub fn compute_fish_layout(area: ratatui::layout::Rect) -> (u16, u16, u16) {
    // Compute the number of lanes based on available terminal height.
    // Each lane uses FISH_HEIGHT rows; ensure at least one lane.
    let lane_height = FISH_HEIGHT;
    let lanes = std::cmp::max(1u16, area.height / lane_height);
    let base_y = area.y;
    (lanes, lane_height, base_y)
}

/// Render all fish inside the fish area.
pub fn compute_fish_render_ops<'a>(
    fishes: &[Fish],
    fish_area: Rect,
    frames_by_species: &'a [(Vec<Text<'a>>, Vec<Text<'a>>)],
    elapsed: Duration,
) -> Vec<(Rect, Text<'a>)> {
    let (_lanes, lane_height, base_y) = compute_fish_layout(fish_area);
    let mut out = Vec::new();

    for fish in fishes.iter() {
        if elapsed.as_millis() < fish.spawn_delay_ms as u128 {
            continue;
        }

        // Select appropriate frames for this fish's species and direction
        let frames_vec = select_frames(frames_by_species, fish.species, fish.facing_right);
        if frames_vec.is_empty() {
            continue;
        }

        // Calculate current animation frame
        let frame_idx = ((elapsed.as_millis() / fish.frame_duration.as_millis()) as usize) % frames_vec.len();
        let fish_text = frames_vec[frame_idx].clone();

        // Compute position, clamping negative X to prevent wrap-around
        let fish_x = fish.x.max(0.0) as u16;
        let right_bound = fish_area.x.saturating_add(fish_area.width);
        let rem_width = right_bound.saturating_sub(fish_x).min(right_bound);
        let fish_h = lane_height.min(fish_area.height.saturating_sub(1));
        let lane_y = base_y.saturating_add(fish.lane as u16 * lane_height) + FISH_Y_OFFSET;

        let fish_render_area = Rect::new(fish_x, lane_y, rem_width, fish_h);
        out.push((fish_render_area, fish_text));
    }

    out
}

pub fn spawn_fishes<R: rand::Rng + ?Sized>(
    rng: &mut R,
    frames_by_species: &[(Vec<Text>, Vec<Text>)],
    screen_width: f32,
    lanes: usize,
) -> Vec<Fish> {
    let mut fishes = Vec::new();
    let spawn_chance = compute_spawn_chance(screen_width);
    let species_count = frames_by_species.len();
    
    for lane in 0..lanes {
        if rng.gen_bool(spawn_chance) {
            let speed = rng.gen_range(2.0..10.0);
            let species = if species_count == 0 { 
                0 
            } else { 
                rng.gen_range(0..species_count) 
            };
            
            let (has_right, has_left) = species_has_directions(frames_by_species, species);
            
            let dir_right = if has_left && has_right {
                rng.gen_bool(0.5)
            } else {
                has_right
            };
            
            let wrap = rng.gen_bool(0.5);
            let spawn_delay_ms = rng.gen_range(0..MAX_SPAWN_DELAY_MS);
            let x = compute_spawn_x(rng, dir_right, screen_width);
            
            fishes.push(Fish {
                lane,
                x,
                vx: if dir_right { speed } else { -speed },
                wrap,
                facing_right: dir_right,
                species,
                frame_duration: Duration::from_millis(DEFAULT_FRAME_DURATION_MS),
                spawn_delay_ms,
            });
        }
    }
    fishes
}

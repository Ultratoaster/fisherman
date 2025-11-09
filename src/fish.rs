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

// Height of a single fish lane (in terminal rows)
pub const FISH_HEIGHT: u16 = 6;

/// Compute layout parameters for fish lanes given the full terminal area.
/// Returns (lanes, lane_height, base_y) where `base_y` is the top row where
/// the grouped fish lanes start.
pub fn compute_fish_layout(area: ratatui::layout::Rect) -> (u16, u16, u16) {
    // For a given fish_group area, lanes stack from the top of that area
    let lanes = 3u16;
    let lane_height = FISH_HEIGHT;
    let base_y = area.y;
    (lanes, lane_height, base_y)
}

/// Render all fish inside the provided `fish_area`. This centralizes how
/// fish lanes are computed and where each fish gets placed.
/// Compute rendering operations for fish: return a Vec of (Rect, Text) where
/// each pair indicates where to render the fish Text.
pub fn compute_fish_render_ops<'a>(
    fishes: &[Fish],
    fish_area: Rect,
    // Frames grouped per-species: Vec of (right_frames, left_frames).
    frames_by_species: &'a [(Vec<Text<'a>>, Vec<Text<'a>>)],
    elapsed: Duration,
) -> Vec<(Rect, Text<'a>)> {
    let (_lanes, lane_height, base_y) = compute_fish_layout(fish_area);
    let mut out = Vec::new();

    for fish in fishes.iter() {
        // Skip rendering until the fish's spawn delay has elapsed
        if elapsed.as_millis() < fish.spawn_delay_ms as u128 {
            continue;
        }
        let species_idx = if frames_by_species.is_empty() {
            0usize
        } else {
            // guard if species index somehow out of range
            fish.species.min(frames_by_species.len() - 1)
        };

        let (ref_vec_right, ref_vec_left) = &frames_by_species[species_idx];
        let frames_vec = if fish.facing_right {
            ref_vec_right.as_slice()
        } else if !ref_vec_left.is_empty() {
            ref_vec_left.as_slice()
        } else {
            ref_vec_right.as_slice()
        };

        if frames_vec.is_empty() {
            continue;
        }

        let frame_idx = ((elapsed.as_millis() / fish.frame_duration.as_millis()) as usize) % frames_vec.len();
        let fish_text = frames_vec[frame_idx].clone();

    // compute placement within the provided fish_area
    // Avoid casting negative positions directly to u16 (which wraps). Clamp
    // negative X to 0 so partially off-screen fish render at the left edge
    // instead of appearing far on the right due to unsigned wrap.
    let fish_x_f = if fish.x.is_sign_negative() { 0.0 } else { fish.x };
    let fish_x = fish_x_f as u16;
    let right_bound = fish_area.x.saturating_add(fish_area.width);
    let rem_width = if fish_x >= right_bound { 0 } else { right_bound.saturating_sub(fish_x) };
        let fish_h = lane_height.min(fish_area.height.saturating_sub(1));
        let lane_y = base_y.saturating_add(fish.lane as u16 * lane_height) + 2;

        let fish_render_area = Rect::new(fish_x, lane_y, rem_width, fish_h);
        out.push((fish_render_area, fish_text));
    }

    out
}

pub fn spawn_fishes<R: rand::Rng + ?Sized>(rng: &mut R, has_left: bool, has_right: bool, species_count: usize, screen_width: f32) -> Vec<Fish> {
    let mut fishes = Vec::new();
    for lane in 0..3usize {
        if rng.gen_bool(0.7) {
            let speed = rng.gen_range(2.0..10.0);
            let dir_right = if has_left && has_right {
                rng.gen_bool(0.5)
            } else if has_right {
                true
            } else {
                false
            };
            let wrap = true;
            let species = if species_count == 0 { 0 } else { rng.gen_range(0..species_count) };
            // random stagger delay up to 5 seconds
            let spawn_delay_ms = rng.gen_range(0..5000u64);
            // choose initial x at the edge depending on movement direction
            let x = if dir_right {
                // start a bit off the left edge
                0.0
            } else {
                // start a bit off the right edge
                screen_width
            };
            fishes.push(Fish {
                lane,
                x,
                vx: if dir_right { speed } else { -speed },
                wrap,
                facing_right: dir_right,
                species,
                frame_duration: Duration::from_millis(150),
                spawn_delay_ms,
            });
        }
    }
    fishes
}

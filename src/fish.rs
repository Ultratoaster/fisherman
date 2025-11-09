use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Fish {
    pub lane: usize,
    pub x: f32,
    pub vx: f32,
    pub wrap: bool,
    pub facing_right: bool,
    pub frame_duration: Duration,
}

// Height of a single fish lane (in terminal rows)
pub const FISH_HEIGHT: u16 = 3;

/// Compute layout parameters for fish lanes given the full terminal area.
/// Returns (lanes, lane_height, base_y) where `base_y` is the top row where
/// the grouped fish lanes start.
pub fn compute_fish_layout(area: ratatui::layout::Rect) -> (u16, u16, u16) {
    let lanes = 3u16;
    let lane_height = FISH_HEIGHT;
    let fish_area_height = lane_height.saturating_mul(lanes) - 2;
    let base_y = if area.height > fish_area_height {
        area.height.saturating_sub(fish_area_height)
    } else {
        0
    };
    (lanes, lane_height, base_y)
}

pub fn spawn_fishes<R: rand::Rng + ?Sized>(rng: &mut R) -> Vec<Fish> {
    let mut fishes = Vec::new();
    for lane in 0..3usize {
        if rng.gen_bool(0.7) {
            let x = rng.gen_range(0.0..30.0);
            let speed = rng.gen_range(0.5..2.0);
            let dir_right = rng.gen_bool(0.5);
            let wrap = rng.gen_bool(0.5);
            fishes.push(Fish {
                lane,
                x,
                vx: if dir_right { speed } else { -speed },
                wrap,
                facing_right: dir_right,
                frame_duration: Duration::from_millis(150),
            });
        }
    }
    fishes
}

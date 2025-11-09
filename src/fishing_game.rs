use rand::Rng;

#[derive(Debug, Clone)]
pub struct CaughtFish {
    pub species_name: String,
    pub size: f32,
    pub size_category: SizeCategory,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SizeCategory {
    Tiny,
    Small,
    Average,
    Large,
    Massive,
}

impl SizeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            SizeCategory::Tiny => "Tiny!",
            SizeCategory::Small => "Small",
            SizeCategory::Average => "Average",
            SizeCategory::Large => "Large",
            SizeCategory::Massive => "Massive!",
        }
    }
}

pub fn generate_fish_size<R: Rng + ?Sized>(rng: &mut R) -> f32 {
    let u1: f32 = rng.gen_range(0.001..1.0);
    let u2: f32 = rng.gen_range(0.0..1.0);
    
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    
    let mean = 50.0;
    let stddev = 15.0;
    let size = mean + z0 * stddev;
    
    size.clamp(1.0, 100.0)
}

pub fn categorize_size(size: f32) -> SizeCategory {
    if size < 20.0 {
        SizeCategory::Tiny
    } else if size < 40.0 {
        SizeCategory::Small
    } else if size < 60.0 {
        SizeCategory::Average
    } else if size < 80.0 {
        SizeCategory::Large
    } else {
        SizeCategory::Massive
    }
}

pub fn check_collision(
    hook_x: u16,
    hook_y: u16,
    fish_x: f32,
    fish_y: u16,
    fish_width: u16,
    fish_height: u16,
) -> bool {
    let fish_left = fish_x as u16;
    let fish_right = fish_left.saturating_add(fish_width);
    let fish_top = fish_y;
    let fish_bottom = fish_y.saturating_add(fish_height);
    
    hook_x >= fish_left && hook_x < fish_right && hook_y >= fish_top && hook_y < fish_bottom
}

impl CaughtFish {
    pub fn new(species_name: String, size: f32) -> Self {
        let size_category = categorize_size(size);
        CaughtFish {
            species_name,
            size,
            size_category,
        }
    }
    
    pub fn format_catch(&self) -> String {
        let article = if self.size_category == SizeCategory::Average {
            "an"
        } else {
            "a"
        };
        format!(
            "You caught {} {} {}!\nSize: {:.1} cm",
            article,
            self.size_category.as_str(),
            self.species_name,
            self.size
        )
    }
}

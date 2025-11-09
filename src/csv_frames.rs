use std::io;
use std::fs;
use std::collections::HashMap;
use serde::Deserialize;
use ratatui::style::Color;
use ratatui::text::{Span, Line, Text};

fn de_hex_to_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let hex = s.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(serde::de::Error::custom("invalid hex color length"));
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(serde::de::Error::custom)?;
    Ok(Color::Rgb(r, g, b))
}

#[derive(Debug, Deserialize)]
struct CellRow {
    #[serde(rename = "X")] pub x: u32,
    #[serde(rename = "Y")] pub y: u32,
    #[serde(rename = "ASCII")] pub ascii: String,
    #[serde(rename = "Foreground", deserialize_with = "de_hex_to_color")] pub foreground: Color,
}

pub fn load_csv_frame(path: &str) -> io::Result<Text<'static>> {
    let content = fs::read_to_string(path)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    let mut cells: HashMap<(u32, u32), (char, (u8, u8, u8))> = HashMap::new();
    let mut max_x = 0;
    let mut max_y = 0;

    for result in reader.deserialize() {
        let row: CellRow = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let x = row.x;
        let y = row.y;
        let ch = row.ascii.chars().next().unwrap_or(' ');

        let fg_rgb = match row.foreground {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (255, 255, 255),
        };

        max_x = max_x.max(x);
        max_y = max_y.max(y);
        cells.insert((x, y), (ch, fg_rgb));
    }

    let mut rows: Vec<Line> = Vec::with_capacity((max_y as usize) + 1);
    for y in 0..=max_y {
        let mut span_row: Vec<Span> = Vec::with_capacity((max_x as usize) + 1);
        for x in 0..=max_x {
            if let Some((ch, fg)) = cells.get(&(x, y)) {
                let styled = Span::styled(
                    ch.to_string(),
                    ratatui::style::Style::default()
                        .fg(Color::Rgb(fg.0, fg.1, fg.2))
                );
                span_row.push(styled);
            } else {
                span_row.push(Span::raw(" "));
            }
        }
        rows.push(Line::from(span_row));
    }

    Ok(Text::from(rows))
}

pub fn load_frames_from_dir(dir: &str) -> io::Result<Vec<Text<'static>>> {
    let mut paths: Vec<std::path::PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|ext| ext == "csv").unwrap_or(false))
        .collect();

    paths.sort_by_key(|p| p.file_name().map(|s| s.to_owned()));

    let mut frames = Vec::with_capacity(paths.len());
    for p in paths {
        let s = p.to_string_lossy().to_string();
        match load_csv_frame(&s) {
            Ok(t) => frames.push(t),
            Err(e) => eprintln!("failed to load {}: {}", s, e),
        }
    }

    Ok(frames)
}

pub type SpeciesFrames = (Vec<Text<'static>>, Vec<Text<'static>>);

#[derive(Debug, Clone)]
pub struct FishSpecies {
    pub name: String,
    pub frames: SpeciesFrames,
}

/// Expected file structure:
/// base_dir/
///   species1/
///     left/*.csv
///     right/*.csv
///   species2/
///     left/*.csv
///     right/*.csv
pub fn load_all_fish_species(base_dir: &str) -> io::Result<Vec<FishSpecies>> {
    let mut per_species: Vec<FishSpecies> = Vec::new();

    let base = std::path::Path::new(base_dir);
    if !base.exists() {
        return Ok(per_species);
    }

    for entry in std::fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() { continue; }

        let species_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let mut right_frames: Vec<Text<'static>> = Vec::new();
        let mut left_frames: Vec<Text<'static>> = Vec::new();

        let right_dir = path.join("right");
        if right_dir.exists() && right_dir.is_dir() {
            if let Ok(mut v) = load_frames_from_dir(right_dir.to_string_lossy().as_ref()) {
                right_frames.append(&mut v);
            }
        }

        let left_dir = path.join("left");
        if left_dir.exists() && left_dir.is_dir() {
            if let Ok(mut v) = load_frames_from_dir(left_dir.to_string_lossy().as_ref()) {
                left_frames.append(&mut v);
            }
        }

        if !right_frames.is_empty() || !left_frames.is_empty() {
            per_species.push(FishSpecies {
                name: species_name,
                frames: (right_frames, left_frames),
            });
        }
    }

    Ok(per_species)
}

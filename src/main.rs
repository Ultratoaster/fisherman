use std::io;
use std::fs;
use std::time::{Duration, Instant};
use std::thread;
use std::collections::HashMap;
use serde::Deserialize;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    style::Color,
    Terminal,
};
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
    #[serde(rename = "Background", deserialize_with = "de_hex_to_color")] pub background: Color,
}


fn load_csv_frame(path: &str) -> io::Result<Text<'static>> {
    let content = fs::read_to_string(path)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    let mut cells: HashMap<(u32, u32), (char, (u8, u8, u8), (u8, u8, u8))> = HashMap::new();
    let mut max_x = 0;
    let mut max_y = 0;

    for result in reader.deserialize() {
        let row: CellRow = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let x = row.x;
        let y = row.y;
    // CSV now stores the actual character (as a 1-char string).
    // Take the first char or default to space if empty.
    let ch = row.ascii.chars().next().unwrap_or(' ');

        let fg_rgb = match row.foreground {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (255, 255, 255),
        };
        let bg_rgb = match row.background {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (0, 0, 0),
        };

        max_x = max_x.max(x);
        max_y = max_y.max(y);
        cells.insert((x, y), (ch, fg_rgb, bg_rgb));
    }

    let mut rows: Vec<Line> = Vec::with_capacity((max_y as usize) + 1);
    for y in 0..=max_y {
        let mut span_row: Vec<Span> = Vec::with_capacity((max_x as usize) + 1);
        for x in 0..=max_x {
            if let Some((ch, fg, bg)) = cells.get(&(x, y)) {
                let styled = Span::styled(
                    ch.to_string(),
                    ratatui::style::Style::default()
                        .fg(Color::Rgb(fg.0, fg.1, fg.2))
                        .bg(Color::Rgb(bg.0, bg.1, bg.2)),
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

fn load_frames_from_dir(dir: &str) -> io::Result<Vec<Text<'static>>> {
    let mut paths: Vec<std::path::PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|ext| ext == "csv").unwrap_or(false))
        .collect();

    // sort by filename to get deterministic ordering
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

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    // Load CSV frames from folder
    let frames = match load_frames_from_dir("src/fisherman_frames") {
        Ok(f) if !f.is_empty() => f,
        Ok(_) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            eprintln!("No CSV frames found in folder");
            return Err(io::Error::new(io::ErrorKind::NotFound, "no frames"));
        }
        Err(e) => {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            eprintln!("Failed to load frames: {}", e);
            return Err(e);
        }
    };

    // Simulated "loading" duration and per-frame display time
    let start = Instant::now();
    let load_time = Duration::from_secs(5);
    let frame_duration = Duration::from_millis(500);

    loop {
        let elapsed = start.elapsed();

        // Once "done" → show fish caught; otherwise cycle frames
        let text = if elapsed >= load_time {
            Text::from("Got one!")
        } else {
            let idx = ((elapsed.as_millis() / frame_duration.as_millis()) as usize) % frames.len();
            frames[idx].clone()
        };

        terminal.draw(|f| {
            let size = f.area();
            let block = Block::default()
                .title("Fisherman")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(text)
                .block(block);
            f.render_widget(paragraph, size);
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

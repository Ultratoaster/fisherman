# 🎣 Fisherman - Terminal Fishing Game

A terminal-based fishing game written in Rust that features fish, ocean waves, and a fishing mechanic. The game can run standalone or be controlled by external applications through three different IPC (Inter-Process Communication) methods.

## What It Does

Fisherman is an interactive terminal game where you:
- Cast a fishing line by holding and releasing the spacebar
- Catch different species of fish (Goby, Goldfish, Shark) that swim across the screen
- Watch animated ocean waves, a twinkling starry sky, and moon
- Control the game from Python applications using stdin, signal files, or named pipes

The game includes multiple integration modes for embedding into other applications, making it suitable for use as a widget or subprocess in larger projects.

## How It Works

### Tech Stack

- **Rust 1.75+**: Core game engine and rendering
- **Ratatui**: Terminal UI framework for rendering sprites and animations
- **Crossterm**: Cross-platform terminal manipulation and input handling
- **Python 3.8+**: Optional helper scripts for IPC integration
- **CSV-based sprites**: All fish and moon graphics stored as CSV files for easy editing

### Implementation

The game uses:
- Frame-based rendering loop with Ratatui's terminal buffer
- CSV sprite loader (`csv_frames.rs`) that converts CSV files into terminal graphics
- Thread-safe IPC signal handling using `Arc<Mutex<>>` for external control
- Cross-platform input detection with fallback logic for Linux spacebar issues
- Configurable fish spawning with species-specific movement patterns

## Requirements

### For Running the Game

**Windows:**
- Rust 1.75 or later (install from https://rustup.rs/)
- No additional dependencies required

**Linux:**
- Rust 1.75 or later
- For portable builds: `x86_64-unknown-linux-musl` target

### For Python Integration (Optional)

- Python 3.8 or later
- For Textual widget integration:
  - `textual` package: `pip install textual`
  - `rich` package: `pip install rich`
- For Windows named pipe control:
  - `pywin32` package: `pip install pywin32`

## Quick Start - Running the Game

### Option 1: Standalone Game (Recommended for First Run)

1. **Build the game:**
   ```cmd
   cargo build --release
   ```

2. **Run the game:**
   ```cmd
   target\release\fisherman.exe
   ```

3. **Play:**
   - Hold **SPACEBAR** to charge your cast (watch the power meter)
   - Release **SPACEBAR** to cast the line
   - Catch fish as they swim by!
   - Press **Q** or **ESC** to quit

### Option 2: Python Subprocess Control (stdin IPC)

1. **Build the game** (if not already built):
   ```cmd
   cargo build --release
   ```

2. **Run the Python control script:**
   ```cmd
   py subprocess_example.py
   ```

This demonstrates controlling the game by sending commands through stdin:
- `SUCCESS:You caught a fish!\n` - Triggers a success catch with message
- `FAILURE:The fish got away!\n` - Triggers a failed catch with message

### Option 3: Signal File Control (Separate Terminal)

1. **Build the game** (if not already built):
   ```cmd
   cargo build --release
   ```

2. **Run the Python launcher script:**
   ```cmd
   py control_in_terminal.py
   ```

This opens the game in a separate terminal window and demonstrates sending signals via a temporary file.

### Option 4: Named Pipe Control (Windows/Unix FIFO)

1. **Build the game** (if not already built):
   ```cmd
   cargo build --release
   ```

2. **Run the Python pipe control script:**
   ```cmd
   py pipe_control.py
   ```

This demonstrates controlling the game through named pipes (Win32 pipes on Windows, FIFO on Linux/macOS).

## Linux Build Instructions

For portable Linux binaries that work across distributions:

1. **Add the musl target:**
   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

2. **Build static binary:**
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```

3. **Run the game:**
   ```bash
   ./target/x86_64-unknown-linux-musl/release/fisherman
   ```

The musl build creates a statically-linked binary with no glibc dependencies, making it portable across different Linux distributions.

## IPC Modes Summary

The game supports three IPC methods for external control:

| Mode | Command | Use Case |
|------|---------|----------|
| **Stdin** | `--subprocess` | Embedded in Python apps with subprocess.PIPE |
| **Signal File** | `--signal-file <path>` | Separate terminal with file-based communication |
| **Named Pipe** | `--pipe <name>` | High-performance IPC with Win32 pipes/FIFO |

### Message Format

All IPC methods accept messages in the format:
```
SUCCESS:Your message here\n
FAILURE:Your message here\n
```

- `SUCCESS` triggers a successful catch animation and displays the message
- `FAILURE` triggers a failed catch animation and displays the message

## Troubleshooting

### Windows: "Python not found" error

Use `py` instead of `python`:
```cmd
py subprocess_example.py
```

### Linux: Spacebar doesn't cast

If releasing spacebar doesn't trigger the cast:
1. Keep holding spacebar until the power meter fills
2. **Press spacebar again** while charging to cast (release detection fallback)

This is a known issue with some Linux terminal emulators not firing key release events reliably.

### Linux: "glibc version GLIBC_X.XX not found"

Use the static musl build instead:
```bash
cargo build --release --target x86_64-unknown-linux-musl
./target/x86_64-unknown-linux-musl/release/fisherman
```

### Named Pipes on Windows Require pywin32

For `pipe_control.py` on Windows:
```cmd
pip install pywin32
```

Linux and macOS use FIFO (named pipes) which are built into the OS.

## Verification Steps for Graders

To verify the application works correctly:

1. **Build Test:**
   ```cmd
   cargo build --release
   ```
   Should complete without errors.

2. **Standalone Run Test:**
   ```cmd
   target\release\fisherman.exe
   ```
   - You should see ocean waves, stars, and a fisherman sprite
   - Hold and release SPACEBAR to cast
   - Press Q to quit

3. **Python Integration Test:**
   ```cmd
   py subprocess_example.py
   ```
   - Game should launch
   - After a few seconds, you should see automated catch messages
   - Game should auto-quit after demo

4. **Linux Build Test** (if testing on Linux):
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ./target/x86_64-unknown-linux-musl/release/fisherman
   ```
   - Should run without glibc errors on any modern Linux distribution

## Additional Documentation

- **SUBPROCESS_MODE.md**: Detailed IPC implementation guide
- **TEXTUAL_INTEGRATION.md**: How to embed as a Textual widget
- **QUICKSTART.md**: Quick reference for common commands

## Project Structure

```
fisherman/
├── src/
│   ├── main.rs              # Entry point, IPC handlers, game loop
│   ├── fishing_game.rs      # Game state and catch logic
│   ├── fisherman.rs         # Fisherman sprite and animations
│   ├── fishing_line.rs      # Casting mechanics and line rendering
│   ├── fish.rs              # Fish spawning and movement
│   ├── ocean.rs             # Wave animations
│   ├── stars.rs             # Star twinkling effects
│   ├── csv_frames.rs        # CSV sprite loader
│   └── fish/                # Fish sprite CSV files
├── subprocess_example.py    # stdin IPC demo
├── control_in_terminal.py   # Signal file IPC demo
├── pipe_control.py          # Named pipe IPC demo
└── Cargo.toml              # Rust dependencies
```

## License

This project was created for a hackathon demonstration.
67
67
67
67
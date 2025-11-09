# Fisherman Game 🎣

A terminal-based fishing game built in Rust with animated ASCII graphics, featuring fish spawning, casting mechanics, and inter-process communication for integration with other applications.

## Features

- 🐟 Multiple fish species with left/right animations loaded from CSV
- 🎣 Realistic fishing mechanics with power meter and casting arc
- 🌊 Animated ocean with foam effects
- ⭐ Twinkling stars in night sky
- 🌙 Moon sprite rendering
- 🎮 Interactive controls for casting and reeling
- 📡 IPC support via stdin, named pipes, or signal files
- 🐍 Python Textual integration examples
- 🔧 Subprocess mode for external application control

## Prerequisites

### For Building

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs/)
  ```bash
  # Linux/macOS/WSL
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  
  # Windows (PowerShell)
  # Download and run rustup-init.exe from rustup.rs
  ```

- **Build Tools**
  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install build-essential pkg-config
  
  # macOS (Xcode Command Line Tools)
  xcode-select --install
  
  # Windows
  # Install Visual Studio Build Tools or Visual Studio Community
  ```

### For Python Integration (Optional)

- **Python 3.7+**
- **Dependencies:**
  ```bash
  pip install textual rich
  
  # Windows only (for named pipes)
  pip install pywin32
  ```

## Building the Project

### Quick Start

```bash
# Clone the repository
git clone https://github.com/Ultratoaster/fisherman.git
cd fisherman

# Build in release mode (optimized)
cargo build --release

# Run the game
./target/release/fisherman        # Linux/macOS
.\target\release\fisherman.exe    # Windows
```

### Build Modes

#### Debug Build (Faster Compilation)
```bash
cargo build

# Binary at: target/debug/fisherman
```

#### Release Build (Optimized Performance)
```bash
cargo build --release

# Binary at: target/release/fisherman
```

### Cross-Platform Builds

#### Linux Static Binary (Portable)

Build a statically-linked binary that works on **any** Linux distribution without dependencies:

```bash
# Add musl target for static linking
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Binary at: target/x86_64-unknown-linux-musl/release/fisherman
```

**Benefits:**
- ✅ Works on Ubuntu, Debian, CentOS, Alpine, any distro
- ✅ No glibc version dependencies
- ✅ Single portable executable
- ✅ Perfect for CI/CD and containers

**Verify static linking:**
```bash
ldd target/x86_64-unknown-linux-musl/release/fisherman
# Should output: "not a dynamic executable"
```

#### Cross-Compile for Linux from Windows/WSL

```bash
# Option 1: Use WSL (Recommended)
wsl
cd /mnt/c/Users/YOUR_USERNAME/path/to/fisherman
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Option 2: Use cargo-zigbuild
cargo install cargo-zigbuild
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

#### macOS Builds

```bash
# Intel Macs
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Apple Silicon (M1/M2/M3/M4)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**Note:** Cross-compiling to macOS from Windows/Linux requires the macOS SDK. Best built on an actual Mac or using GitHub Actions with macOS runners.

## Running the Game

### Standalone Mode

The simplest way to play:

```bash
# Run the game
./target/release/fisherman

# Or on Windows
.\target\release\fisherman.exe
```

**Game Controls:**
- **SPACEBAR (hold)** - Charge casting power
- **SPACEBAR (release or press again)** - Cast the line
  - On Linux: Press space again if key release doesn't fire
- **↑ Up Arrow** - Raise hook / Reel in
- **↓ Down Arrow** - Lower hook deeper
- **s** - Send test SUCCESS signal (standalone mode only)
- **f** - Send test FAILURE signal (standalone mode only)
- **q** - Quit game

### Subprocess Modes

The game supports **three IPC methods** for integration with external applications:

#### 1. Stdin Communication (Cross-Platform, Simplest)

Read signals from standard input:

```bash
./target/release/fisherman --subprocess
```

**Send signals via stdin:**
```bash
# From Python
process.stdin.write("SUCCESS:You caught a fish!\n")
process.stdin.flush()

# From shell
echo "SUCCESS:Great catch!" | ./target/release/fisherman --subprocess
```

**Example Python script:**
```bash
python subprocess_example.py
```

#### 2. Signal File (Cross-Platform, File-Based)

Poll a file for signals (good for separate terminal windows):

```bash
# Linux/macOS
./target/release/fisherman --signal-file /tmp/fish_signal.txt

# Windows
.\target\release\fisherman.exe --signal-file C:\Temp\fish_signal.txt
```

**Send signals by writing to file:**
```bash
# Linux/macOS
echo "SUCCESS:Perfect!" > /tmp/fish_signal.txt
echo "FAILURE:Try again!" > /tmp/fish_signal.txt

# Windows (PowerShell)
"SUCCESS:Great job!" | Out-File -FilePath C:\Temp\fish_signal.txt -Encoding ASCII
```

**Example Python script (opens game in separate terminal):**
```bash
python control_in_terminal.py
```

#### 3. Named Pipe (Best Performance, Platform-Specific)

Use named pipes for real-time, efficient communication:

**Windows:**
```bash
# Game connects to existing pipe
.\target\release\fisherman.exe --pipe \\.\pipe\fisherman_signals
```

**Linux/macOS:**
```bash
# Create FIFO (named pipe)
mkfifo /tmp/fisherman_pipe

# Run game
./target/release/fisherman --pipe /tmp/fisherman_pipe

# Send signals from another terminal
echo "SUCCESS:You won!" > /tmp/fisherman_pipe
echo "FAILURE:Better luck next time!" > /tmp/fisherman_pipe
```

**Example Python script (handles pipe creation):**
```bash
# Windows (requires pywin32)
python pipe_control.py

# Linux/macOS
python3 pipe_control.py
```

### Signal Format

All IPC methods accept the same signal format:

```
SUCCESS:Your custom message here
FAILURE:Your custom message here
```

**Visual Effects:**
- `SUCCESS:` - Shows green message box, fisherman kicks, exclamation mark
- `FAILURE:` - Shows red message box, no kick, exclamation mark
- Game exits 3 seconds after receiving any signal

## Python Integration

### Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Run an example
python subprocess_example.py          # Inline mode
python control_in_terminal.py         # Separate terminal
python pipe_control.py                # Named pipes (best performance)
```

### Available Python Scripts

| Script | Mode | Platform | Description |
|--------|------|----------|-------------|
| `subprocess_example.py` | stdin | All | Interactive stdin control (inline) |
| `control_in_terminal.py` | signal file | All | Control from separate terminal window |
| `pipe_control.py` | named pipe | All | Real-time pipe communication |
| `example_app_windows.py` | Textual widget | Windows | Embed game in Textual TUI |
| `example_app.py` | Textual widget (PTY) | Linux/macOS | PTY-based Textual integration |
| `open_in_terminal.py` | launch only | All | Just open game in new window |

### Embedding in Textual Applications

**Windows:**
```python
from fisherman_widget_windows import FishermanWidget

class MyApp(App):
    def compose(self):
        yield FishermanWidget(executable_path="target/release/fisherman.exe")
    
    def on_some_event(self):
        widget = self.query_one(FishermanWidget)
        widget.send_success("Player won!")
```

**Linux/macOS (PTY version):**
```python
from fisherman_widget import FishermanWidget

class MyApp(App):
    def compose(self):
        yield FishermanWidget(executable_path="target/release/fisherman")
```

Run the examples:
```bash
# Windows
py example_app_windows.py

# Linux/macOS
python3 example_app.py
```

## Development

### Project Structure

```
fisherman/
├── src/
│   ├── main.rs              # Main game loop, rendering, IPC handling
│   ├── fish.rs              # Fish spawning, movement, collision
│   ├── fisherman.rs         # Fisherman sprite and kick animation
│   ├── fishing_line.rs      # Casting mechanics, line states
│   ├── fishing_game.rs      # Catch detection, fish size generation
│   ├── ocean.rs             # Ocean waves and foam animation
│   ├── stars.rs             # Star field twinkling effects
│   ├── widgets.rs           # Dock widget rendering
│   └── csv_frames.rs        # CSV sprite loader
├── src/fish/                # Fish sprite data (CSV format)
│   ├── Goby/
│   │   └── left/           # Left-facing Goby frames
│   ├── Goldfish/
│   │   └── left/           # Left-facing Goldfish frames
│   └── Shark/
│       └── right/          # Right-facing Shark frames
├── moon.csv                 # Moon sprite data
├── *.py                     # Python integration examples
├── Cargo.toml              # Rust dependencies
├── requirements.txt        # Python dependencies
└── README.md               # This file
```

### Adding New Fish Species

1. **Create species directory:**
   ```bash
   mkdir -p src/fish/YourFish/left
   mkdir -p src/fish/YourFish/right  # Optional
   ```

2. **Create CSV frame files:**
   - Name format: `frame1.csv`, `frame2.csv`, etc.
   - CSV structure:
     ```csv
     Character,Foreground,Background
     ~,Cyan,
     o,Yellow,
     ^,White,Blue
     ```

3. **CSV Columns:**
   - `Character` - The ASCII character to display
   - `Foreground` - Text color (e.g., `Red`, `Blue`, `Cyan`, `Yellow`, `Green`, etc.)
   - `Background` - Background color (optional, leave empty for transparent)

4. **Game automatically detects new species** on startup

### Building for Distribution

```bash
# Build optimized binary
cargo build --release

# Strip debug symbols (smaller binary)
strip target/release/fisherman  # Linux/macOS

# Create distribution package
mkdir fisherman-v1.0
cp target/release/fisherman fisherman-v1.0/
cp -r src/fish fisherman-v1.0/
cp moon.csv fisherman-v1.0/
cp README.md fisherman-v1.0/
tar -czf fisherman-v1.0-linux.tar.gz fisherman-v1.0/
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Debugging

```bash
# Run with backtrace on panic
RUST_BACKTRACE=1 cargo run

# Run with debug logs
RUST_LOG=debug cargo run

# Check code without building
cargo check
```

## Troubleshooting

### "glibc version mismatch" on Linux

**Solution:** Build with musl for static linking
```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

### Spacebar doesn't work on Linux

**Issue:** Some Linux terminals don't fire key release events reliably.

**Solution:** Press space again to cast (no need to hold). The game detects if you're charging and casts immediately on second press.

### Terminal window doesn't close on Windows

**Issue:** Using `/k` flag keeps window open.

**Solution:** Update `control_in_terminal.py` to use `/c` instead:
```python
cmd = f'start "Fisherman Game" cmd /c "target\\release\\fisherman.exe ..."'
```

### Python can't find module 'textual'

**Solution:** Install dependencies:
```bash
pip install textual rich

# On Windows for pipe_control.py
pip install pywin32
```

### "python: command not found"

**Windows:** Use `py` instead of `python`:
```bash
py subprocess_example.py
```

**Linux/macOS:** Use `python3`:
```bash
python3 subprocess_example.py
```

### Cross-compilation fails on Windows

**Solution:** Use WSL for Linux builds:
```bash
wsl
cd /mnt/c/Users/YOUR_USERNAME/path/to/fisherman
cargo build --release --target x86_64-unknown-linux-musl
```

## Documentation

- **[SUBPROCESS_MODE.md](SUBPROCESS_MODE.md)** - Detailed IPC integration guide
- **[TEXTUAL_INTEGRATION.md](TEXTUAL_INTEGRATION.md)** - Textual widget usage
- **[QUICKSTART.md](QUICKSTART.md)** - Quick reference guide

## License

MIT License - See [LICENSE](LICENSE) file for details

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Ratatui](https://ratatui.rs/) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [Textual](https://textual.textualize.io/) - Python TUI framework (for examples)

---

**Happy Fishing! 🎣**

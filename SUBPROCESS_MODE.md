# Subprocess Mode - Python Integration

The fisherman game can run as a subprocess and receive signals via **stdin**. This is the most portable and standard method for subprocess communication.

## How It Works

When you run the game with the `--subprocess` flag, it:
1. Spawns a background thread to read from stdin
2. Listens for lines in the format: `SUCCESS:message` or `FAILURE:message`
3. Triggers visual responses (exclamation mark, fisherman kick animation)
4. Exits after displaying the signal for 3 seconds

## Usage

### Building

```bash
cargo build --release
```

### Running as Subprocess from Python

```python
import subprocess

# Start the game
process = subprocess.Popen(
    ["target/release/fisherman.exe", "--subprocess"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
    bufsize=1,
)

# Send a SUCCESS signal
process.stdin.write("SUCCESS:You won!\n")
process.stdin.flush()

# Send a FAILURE signal
process.stdin.write("FAILURE:Try again!\n")
process.stdin.flush()

# Wait for game to end
process.wait()
```

### Running the Example

```bash
# Terminal 1: Run the example script
py subprocess_example.py

# The script will prompt you for commands:
# s - Send SUCCESS signal
# f - Send FAILURE signal  
# q - Quit
```

## Signal Format

Signals are sent as text lines via stdin:

- **SUCCESS**: `SUCCESS:Your message here\n`
- **FAILURE**: `FAILURE:Your message here\n`

The message part is currently stored but not displayed. The game shows:
- An exclamation mark (`!`) above the fisherman when a signal is received
- A kick animation if the signal is SUCCESS
- No kick animation if the signal is FAILURE

## Integration in Your Application

### Basic Integration

```python
import subprocess

process = subprocess.Popen(
    ["fisherman", "--subprocess"],
    stdin=subprocess.PIPE,
    text=True,
)

# Send signals based on your application logic
if user_won_game():
    process.stdin.write("SUCCESS:Victory!\n")
    process.stdin.flush()
else:
    process.stdin.write("FAILURE:Defeat!\n")
    process.stdin.flush()
```

### Non-blocking Integration

```python
import subprocess
import threading
import queue

signal_queue = queue.Queue()

def run_game():
    process = subprocess.Popen(
        ["fisherman", "--subprocess"],
        stdin=subprocess.PIPE,
        text=True,
    )
    
    while True:
        signal = signal_queue.get()
        if signal is None:
            break
        process.stdin.write(f"{signal}\n")
        process.stdin.flush()
    
    process.wait()

# Start game in background thread
game_thread = threading.Thread(target=run_game, daemon=True)
game_thread.start()

# Send signals from main thread
signal_queue.put("SUCCESS:Level complete!")
```

## Why Stdin Instead of Signal Files?

**Stdin** is the standard approach for subprocess communication:
- ✅ Works on all platforms (Windows, Linux, macOS)
- ✅ No file I/O overhead
- ✅ Automatic cleanup (no leftover files)
- ✅ Simpler implementation
- ✅ Follows Unix philosophy

**Signal files** (the alternative):
- Requires file system access
- Needs manual cleanup
- Potential race conditions
- More complex error handling

## Textual Integration

If you want to embed the game in a Textual TUI application, you can still use this approach. The game handles its own terminal rendering, and you send signals via stdin based on user actions in your Textual app.

For a full Textual widget implementation, see `fisherman_widget_windows.py` which wraps the subprocess in a widget.

## Debugging

Enable debug output to see when signals are received:

```rust
// In main.rs, add to the signal reading thread:
println!("Received signal: {}", line);
```

Then redirect stderr to see the output:

```bash
py subprocess_example.py 2> debug.log
```

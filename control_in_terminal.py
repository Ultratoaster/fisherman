"""
Example: Running fisherman in a separate terminal with signal control
Uses a signal file for communication so the game can be in its own window.
"""
import subprocess
import time
import sys
import tempfile
from pathlib import Path

def main():
    # Create a temporary signal file
    signal_file = Path(tempfile.gettempdir()) / "fisherman_signals.txt"
    signal_file.write_text("")  # Create empty file
    
    print(f"Signal file: {signal_file}")
    print()
    
    # Start the game in a new terminal window
    if sys.platform == "win32":
        # Windows: Use /c instead of /k to close window when game exits
        cmd = f'start "Fisherman Game" cmd /c "target\\release\\fisherman.exe --signal-file {signal_file}"'
        subprocess.Popen(cmd, shell=True)
        print("Game opened in separate window!")
        print("(Window will close automatically when game exits)")
        
    elif sys.platform == "darwin":
        # macOS: Terminal.app closes automatically when process exits
        subprocess.Popen([
            "open", "-a", "Terminal.app",
            "target/release/fisherman", "--signal-file", str(signal_file)
        ])
        print("Game opened in separate Terminal window!")
        print("(Window will close automatically when game exits)")
        
    else:
        # Linux: Use -e flag which closes terminal when process exits
        subprocess.Popen([
            "x-terminal-emulator", "-e",
            "target/release/fisherman", "--signal-file", str(signal_file)
        ])
        print("Game opened in separate terminal!")
        print("(Window will close automatically when game exits)")
    
    print()
    print("Control the game from here:")
    print("  s - Send SUCCESS signal")
    print("  f - Send FAILURE signal")
    print("  q - Quit")
    print()
    
    try:
        while True:
            command = input("Enter command (s/f/q): ").strip().lower()
            
            if command == 'q':
                print("Closing control script...")
                if signal_file.exists():
                    signal_file.unlink()
                break
                
            elif command == 's':
                signal_file.write_text("SUCCESS:Waargh!")
                print(f"✓ Sent SUCCESS signal (file contains: {signal_file.read_text()!r})")
                time.sleep(0.2)  # Give game time to read
                
            elif command == 'f':
                signal_file.write_text("FAILURE:Graaagh!")
                print(f"✗ Sent FAILURE signal (file contains: {signal_file.read_text()!r})")
                time.sleep(0.2)
                
            else:
                print("Invalid command")
                
    except KeyboardInterrupt:
        print("\nClosing control script...")
        if signal_file.exists():
            signal_file.unlink()

if __name__ == "__main__":
    main()

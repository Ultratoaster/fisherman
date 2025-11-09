"""
Example: Running fisherman in a separate terminal window
This opens the game in its own terminal while allowing signal control from Python.
"""
import subprocess
import time
import sys
import os

def main():
    print("Starting fisherman in a separate terminal window...")
    print()
    
    # Windows: Use 'start' command to open in new terminal
    # The /wait flag makes it wait for the process to complete
    if sys.platform == "win32":
        # Create a named pipe or use a temporary signal file
        # For simplicity, we'll use keyboard input in the main terminal
        
        # Start in a new window without waiting
        process = subprocess.Popen(
            ["start", "cmd", "/k", "target\\release\\fisherman.exe", "--subprocess"],
            shell=True,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        
        print("Game opened in separate window!")
        print("Note: stdin signals won't work with separate terminal.")
        print("Use the in-game test keys instead:")
        print("  Press 's' in game window for SUCCESS")
        print("  Press 'f' in game window for FAILURE")
        print("  Press 'q' in game window to quit")
        
    elif sys.platform == "darwin":
        # macOS: Use 'open' with Terminal.app
        subprocess.Popen([
            "open", "-a", "Terminal.app",
            "target/release/fisherman", "--subprocess"
        ])
        print("Game opened in separate Terminal window!")
        
    else:
        # Linux: Try various terminal emulators
        terminals = [
            ["x-terminal-emulator", "-e"],
            ["gnome-terminal", "--"],
            ["xterm", "-e"],
            ["konsole", "-e"],
        ]
        
        for terminal_cmd in terminals:
            try:
                subprocess.Popen(terminal_cmd + ["target/release/fisherman", "--subprocess"])
                print(f"Game opened in {terminal_cmd[0]}!")
                break
            except FileNotFoundError:
                continue
        else:
            print("No suitable terminal emulator found.")
            return
    
    print()
    print("The game is running in its own window.")
    print("Press Enter here to close this script (game will continue)...")
    input()

if __name__ == "__main__":
    main()

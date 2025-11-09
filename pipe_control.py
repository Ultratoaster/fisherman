"""
Example: Running fisherman with named pipe communication (Windows)
Named pipes provide better IPC than polling files.
"""
import subprocess
import sys
import time
import random

if sys.platform == "win32":
    import win32pipe
    import win32file
    import pywintypes
    
    def main():
        pipe_name = r'\\.\pipe\fisherman_signals'
        
        print(f"Creating named pipe: {pipe_name}")
        print()
        
        # Create the named pipe
        try:
            pipe = win32pipe.CreateNamedPipe(
                pipe_name,
                win32pipe.PIPE_ACCESS_DUPLEX,
                win32pipe.PIPE_TYPE_MESSAGE | win32pipe.PIPE_READMODE_MESSAGE | win32pipe.PIPE_WAIT,
                1, 65536, 65536, 0, None
            )
        except pywintypes.error as e:
            print(f"Failed to create pipe: {e}")
            return
        
        # Start the game in a new terminal
        cmd = f'start "Fisherman Game" cmd /c "target\\release\\fisherman.exe --pipe {pipe_name}"'
        subprocess.Popen(cmd, shell=True)
        print("Game opened in separate window!")
        print("Waiting for game to connect to pipe...")
        
        try:
            # Wait for the game to connect
            win32pipe.ConnectNamedPipe(pipe, None)
            print("Game connected to pipe!")
            print()
            print("Control the game:")
            print("  s - Send SUCCESS signal")
            print("  f - Send FAILURE signal")
            print("  q - Quit")
            print()
            
            while True:
                command = input("Enter command (s/f/q): ").strip().lower()
                
                if command == 'q':
                    print("Closing...")
                    break
                    
                elif command == 's':
                    message = "SUCCESS:Great job!\n"
                    win32file.WriteFile(pipe, message.encode())
                    print("✓ Sent SUCCESS signal")
                    
                elif command == 'f':
                    message = "FAILURE:Try again!\n"
                    win32file.WriteFile(pipe, message.encode())
                    print("✗ Sent FAILURE signal")
                    
                else:
                    print("Invalid command")
                    
        except KeyboardInterrupt:
            print("\nClosing...")
        finally:
            win32file.CloseHandle(pipe)
            print("Pipe closed")

else:
    # Unix/Linux version using FIFO
    import os
    
    def main():
        pipe_path = '/tmp/fisherman_pipe'
        
        # Remove old pipe if it exists
        if os.path.exists(pipe_path):
            os.remove(pipe_path)
        
        # Create FIFO (named pipe)
        os.mkfifo(pipe_path)
        print(f"Created named pipe: {pipe_path}")
        print()
        
        # Start the game in a new terminal
        subprocess.Popen([
            'x-terminal-emulator', '-e',
            'target/release/fisherman', '--pipe', pipe_path
        ])
        print("Game opened in separate terminal!")
        print()
        print("Control the game:")
        print("  s - Send SUCCESS signal")
        print("  f - Send FAILURE signal")
        print("  q - Quit")
        print()
        
        try:
            # Open pipe for writing
            pipe = open(pipe_path, 'w')
            
            while True:
                command = input("Enter command (s/f/q): ").strip().lower()
                
                if command == 'q':
                    print("Closing...")
                    break
                    
                elif command == 's':
                    pipe.write("SUCCESS:Great job!\n")
                    pipe.flush()
                    print("✓ Sent SUCCESS signal")
                    
                elif command == 'f':
                    pipe.write("FAILURE:Try again!\n")
                    pipe.flush()
                    print("✗ Sent FAILURE signal")
                    
                else:
                    print("Invalid command")
                    
        except KeyboardInterrupt:
            print("\nClosing...")
        finally:
            pipe.close()
            if os.path.exists(pipe_path):
                os.remove(pipe_path)
            print("Pipe closed")

if __name__ == "__main__":
    if sys.platform == "win32":
        try:
            import win32pipe
            import win32file
            import pywintypes
            main()
        except ImportError:
            print("ERROR: pywin32 is required for named pipes on Windows")
            print("Install with: pip install pywin32")
    else:
        main()

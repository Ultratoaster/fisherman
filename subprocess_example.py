"""
Example: Running fisherman as a subprocess with stdin signal communication
This is the simplest and most portable approach for Python integration.
"""
import subprocess
import time
import sys

def main():
    # Start the fisherman game in subprocess mode
    # The --subprocess flag enables stdin signal reading
    process = subprocess.Popen(
        ["target/release/fisherman.exe", "--subprocess"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,  # Line buffered
    )
    
    print("Fisherman game started as subprocess")
    print("Commands:")
    print("  s - Send SUCCESS signal")
    print("  f - Send FAILURE signal")
    print("  q - Quit")
    print()
    
    try:
        while True:
            command = input("Enter command (s/f/q): ").strip().lower()
            
            if command == 'q':
                print("Terminating game...")
                process.terminate()
                break
            elif command == 's':
                # Send SUCCESS signal via stdin
                process.stdin.write("SUCCESS:Great job!\n")
                process.stdin.flush()
                print("Sent SUCCESS signal")
            elif command == 'f':
                # Send FAILURE signal via stdin
                process.stdin.write("FAILURE:Try again!\n")
                process.stdin.flush()
                print("Sent FAILURE signal")
            else:
                print("Invalid command")
            
            # Check if process is still running
            if process.poll() is not None:
                print("Game has ended")
                break
                
    except KeyboardInterrupt:
        print("\nTerminating game...")
        process.terminate()
    finally:
        process.wait()
        print("Game process ended")

if __name__ == "__main__":
    main()

# focused-window-volume
(Windows) Remap your keyboard volume controls to target your focused window instead of system volume

## Build Instructions

### Prerequisites
- Install [Rust](https://rustup.rs/) (latest stable version)
- Windows 10 or later
- Visual Studio Build Tools (will be automatically prompted during cargo build if missing)

### Building from Source
1. Clone the repository:
   ```
   git clone https://github.com/easymac/focused-window-volume.git
   cd focused-window-volume
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run the application:
   ```
   cargo run --release
   ```

## Installation
The application will be built in the `target/release` directory. You can:

1. Copy the executable (`focused-window-volume.exe`) to a permanent location
2. Create a shortcut to the application in your Windows startup folder to have it run on system boot:
   - Press `Win+R`, type `shell:startup` and press Enter
   - Create a shortcut to the executable in this folder

### Usage
- Volume keys should automatically be captured once the application is running
- To exit the application, right-click the system tray icon and select "Exit"


## Implementation notes

For applications that use multi-process achitecture (e.g. Google Chrome), the process (and PID) which is associated with a window will often be different from the process (and PID) which is associated with a session.

As a heuristic, the path of the executable is used to identify application groups instead of process IDs in order to match the focused window with the audio session.

For now, we're going to evaluate the practical reliability of this heuristic and then consider other approaches if there are problems.

All suggested solutions are very welcome & appreciated to be filed as issues.

## Contributing

Thank you for thinking about helping out with the project!

All bug reports, suggestions, feedback, and ideas are welcome & appreciated as Issues. Pull requests are very welcome!

Code review PRs are especially welcome 🔥


## License
[MIT License](LICENSE)


mod tray;
mod keyboard;
mod focus;
mod audio;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install keyboard hook to capture volume keys
    keyboard::install_keyboard_hook()?;
    
    // Set up system tray
    let tray = tray::Tray::new("Focused Window Volume")?;
    println!("Tray application started. Check your system tray!");
    
    // Run the message loop - this keeps the application running
    let result = tray.run();
    
    // Cleanup keyboard hook before exiting
    keyboard::uninstall_keyboard_hook()?;
    
    // Return any result from the tray
    result
}

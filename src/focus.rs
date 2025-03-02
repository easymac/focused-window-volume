// Removed unused import: use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
// Removing incorrect import and using direct imports from Threading
// use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;

pub fn get_focused_window() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get handle to the foreground window
        let hwnd = GetForegroundWindow();
        
        if hwnd.0 == 0 {
            println!("No window currently has focus");
            return Ok(());
        }
        
        // Get the window title
        let mut title_buffer = [0u16; 512];
        let length = GetWindowTextW(hwnd, &mut title_buffer);
        
        if length > 0 {
            let title = OsString::from_wide(&title_buffer[..length as usize])
                .to_string_lossy()
                .to_string();
            
            println!("Focused window: {}", title);
            println!("Window handle: {:?}", hwnd);
        } else {
            println!("Found window with no title, handle: {:?}", hwnd);
        }
        
        Ok(())
    }
}

pub fn get_focused_window_pid() -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        // Get handle to the foreground window
        let hwnd = GetForegroundWindow();

        if hwnd.0 == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No window currently has focus"
            )));
        }

        // Get the process ID of the window
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        Ok(pid)
    }
}

pub fn get_focused_window_details() -> Result<(u32, String), Box<dyn std::error::Error>> {
    unsafe {
        // Get handle to the foreground window
        let hwnd = GetForegroundWindow();

        if hwnd.0 == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No window currently has focus"
            )));
        }

        // Get the process ID of the window
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        // Get process executable path
        let process_path = get_process_path(pid)?;
        
        Ok((pid, process_path))
    }
}

pub fn get_process_path(pid: u32) -> Result<String, Box<dyn std::error::Error>> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;
        
        let mut buffer = [0u16; 260]; // MAX_PATH
        let mut size = buffer.len() as u32;
        
        QueryFullProcessImageNameW(
            handle, 
            PROCESS_NAME_FORMAT(0), 
            PWSTR(buffer.as_mut_ptr()), 
            &mut size
        )?;
        CloseHandle(handle);
        
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        
        Ok(path)
    }
}

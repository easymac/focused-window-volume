use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr::null_mut;
use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::audio;
use crate::focus;

static HOOK_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

// Virtual key codes for media keys - using u32 to match KBDLLHOOKSTRUCT.vkCode type
const VK_VOLUME_MUTE: u32 = 0xAD;
const VK_VOLUME_DOWN: u32 = 0xAE;
const VK_VOLUME_UP: u32 = 0xAF;

// Callback function for keyboard hook
extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code >= 0 {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            
            // Check if it's a volume key event
            match kb_struct.vkCode {
                VK_VOLUME_UP => {
                    if wparam.0 == WM_KEYDOWN as usize {
                        println!("Volume Up key pressed");

                        match focus::get_focused_window_details() {
                            Ok((pid, process_path)) => {
                                match audio::get_session_by_process_path(&process_path) {
                                    Ok(session) => {
                                        println!("Session: {:?}", session);
                                        match audio::get_session_volume(&session) {
                                            Ok(volume) => {
                                                println!("Volume: {}", volume);
                                                let new_volume = volume + 0.02;
                                                let new_volume = if new_volume > 1.0 { 1.0 } else { new_volume };
                                                match audio::set_session_volume(&session, new_volume) {
                                                    Ok(_) => println!("Volume set to: {}", new_volume),
                                                    Err(e) => println!("Error setting volume: {:?}", e)
                                                }
                                            }
                                            Err(e) => println!("Error getting session volume: {:?}", e)
                                        }
                                    }
                                    Err(e) => println!("Error getting audio session: {:?}", e)
                                }
                            }
                            Err(e) => {
                                println!("Error getting focused window details: {:?}", e);
                            }
                        }
                        return LRESULT(1); // Prevent default behavior
                    }
                },
                VK_VOLUME_DOWN => {
                    if wparam.0 == WM_KEYDOWN as usize {
                        println!("Volume Down key pressed");
                        match focus::get_focused_window_details() {
                            Ok((pid, process_path)) => {
                                match audio::get_session_by_process_path(&process_path) {
                                    Ok(session) => {
                                        println!("Session: {:?}", session);
                                        match audio::get_session_volume(&session) {
                                            Ok(volume) => {
                                                println!("Volume: {}", volume);
                                                let new_volume = volume - 0.02;
                                                let new_volume = if new_volume < 0.0 { 0.0 } else { new_volume };
                                                match audio::set_session_volume(&session, new_volume) {
                                                    Ok(_) => println!("Volume set to: {}", new_volume),
                                                    Err(e) => println!("Error setting volume: {:?}", e)
                                                }
                                            }
                                            Err(e) => println!("Error getting session volume: {:?}", e)
                                        }
                                    }
                                    Err(e) => println!("Error getting audio session: {:?}", e)
                                }
                            }
                            Err(e) => {
                                println!("Error getting focused window details: {:?}", e);
                            }
                        }
                        return LRESULT(1); // Prevent default behavior
                    }
                },
                VK_VOLUME_MUTE => {
                    if wparam.0 == WM_KEYDOWN as usize {
                        println!("Volume Mute key pressed");
                        // In the future, we'll handle mute here
                        return LRESULT(1); // Prevent default behavior
                    }
                },
                _ => {}
            }
        }
        
        // Call the next hook in the chain for non-volume keys
        CallNextHookEx(HHOOK(HOOK_HANDLE.load(Ordering::SeqCst) as isize), code, wparam, lparam)
    }
}

pub fn install_keyboard_hook() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            HINSTANCE(0),
            0,
        )?;
        
        // Store the hook handle
        HOOK_HANDLE.store(hook.0 as *mut c_void, Ordering::SeqCst);
        println!("Keyboard hook installed successfully. Listening for volume keys...");
        
        Ok(())
    }
}

pub fn uninstall_keyboard_hook() -> Result<(), Box<dyn std::error::Error>> {
    let hook_ptr = HOOK_HANDLE.load(Ordering::SeqCst);
    if !hook_ptr.is_null() {
        unsafe {
            UnhookWindowsHookEx(HHOOK(hook_ptr as isize))?;
            HOOK_HANDLE.store(null_mut(), Ordering::SeqCst);
            println!("Keyboard hook uninstalled");
        }
    }
    Ok(())
} 
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
                        handle_volume_up();
                        return LRESULT(1); // Prevent default behavior
                    }
                },
                VK_VOLUME_DOWN => {
                    if wparam.0 == WM_KEYDOWN as usize {
                        handle_volume_down();
                        return LRESULT(1); // Prevent default behavior
                    }
                },
                VK_VOLUME_MUTE => {
                    if wparam.0 == WM_KEYDOWN as usize {
                        handle_volume_mute();
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


fn handle_volume_up() {
    match focus::get_focused_window_session() {
        Ok(session) => {
            audio::increment_session_volume(&session, 0.02).unwrap();
        }
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}

fn handle_volume_down() {
    match focus::get_focused_window_session() {
        Ok(session) => {
            audio::decrement_session_volume(&session, 0.02).unwrap();
        }
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}

fn handle_volume_mute() {
    match focus::get_focused_window_session() {
        Ok(session) => audio::toggle_session_mute(&session).unwrap(),
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}
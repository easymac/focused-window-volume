use std::sync::atomic::{AtomicPtr, AtomicU64, AtomicUsize, Ordering};
use std::ptr::null_mut;
use std::ffi::c_void;
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use crate::audio;
use crate::focus;
use std::sync::Mutex;

static HOOK_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

// Thread-safe implementation using Mutex
lazy_static::lazy_static! {
    static ref VOLUME_STATE: Mutex<VolumeKeyState> = Mutex::new(VolumeKeyState {
        last_pressed: None,
        base_increment: 0.01,
        time_deltas: [1000; 5], // Initialize with 1000ms (1 second)
        current_index: 0,
        history_size: 0,
        max_history_size: 5,
        max_acceleration: 15.0,
        min_acceleration: 1.0,
        decay_rate: 0.016,
    });
}

// Struct to track volume key state
struct VolumeKeyState {
    last_pressed: Option<Instant>,
    base_increment: f32,
    // Fixed-size array for storing time deltas (in milliseconds)
    time_deltas: [u64; 5],
    current_index: usize,
    history_size: usize,
    max_history_size: usize,
    max_acceleration: f32,
    min_acceleration: f32,
    decay_rate: f32,
}

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


fn handle_volume_mute() {
    match focus::get_focused_window_session() {
        Ok(session) => audio::toggle_session_mute(&session).unwrap(),
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}


fn calculate_volume_adjustment() -> f32 {
    let now = Instant::now();
    
    // Use a mutex to safely access our state
    let mut state = VOLUME_STATE.lock().unwrap();
    
    // Calculate time since last press
    let elapsed = if let Some(last) = state.last_pressed {
        now.duration_since(last)
    } else {
        Duration::from_secs(1)
    };
    
    // Update last pressed time
    state.last_pressed = Some(now);
    
    // Convert to milliseconds
    let elapsed_ms = elapsed.as_millis() as u64;
    
    // Get the current index first, to avoid the borrow conflict
    let current_idx = state.current_index;
    
    // Update our circular buffer of time deltas
    state.time_deltas[current_idx] = elapsed_ms;
    state.current_index = (current_idx + 1) % state.max_history_size;
    if state.history_size < state.max_history_size {
        state.history_size += 1;
    }
    
    // Calculate average time delta
    let avg_elapsed_ms = if state.history_size == 0 {
        elapsed_ms
    } else {
        let sum: u64 = state.time_deltas.iter().take(state.history_size).sum();
        sum / state.history_size as u64
    };
    
    // Use the parameters from the state instead of hardcoded values
    let acceleration_factor = state.max_acceleration * (-state.decay_rate * avg_elapsed_ms as f32).exp();
    let acceleration_factor = acceleration_factor.max(state.min_acceleration).min(state.max_acceleration);
    
    // Debug output
    println!("Time delta: {}ms, Avg delta: {}ms, Acceleration: {:.2}x", 
             elapsed_ms, avg_elapsed_ms, acceleration_factor);
    
    // Return the adjusted increment
    state.base_increment * acceleration_factor
}

fn handle_volume_up() {
    match focus::get_focused_window_session() {
        Ok(session) => {
            // Get the current volume
            let current_volume = match audio::get_session_volume(&session) {
                Ok(vol) => vol,
                Err(e) => {
                    println!("Error getting volume: {:?}", e);
                    return;
                }
            };

            // Calculate adaptive adjustment
            let adjustment = calculate_volume_adjustment();
            let new_volume = (current_volume + adjustment).clamp(0.0, 1.0);

            // Set the new volume
            if let Err(e) = audio::set_session_volume(&session, new_volume) {
                println!("Error setting volume: {:?}", e);
            }
        }
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}

fn handle_volume_down() {
    match focus::get_focused_window_session() {
        Ok(session) => {
            // Get the current volume
            let current_volume = match audio::get_session_volume(&session) {
                Ok(vol) => vol,
                Err(e) => {
                    println!("Error getting volume: {:?}", e);
                    return;
                }
            };

            // Calculate adaptive adjustment
            let adjustment = calculate_volume_adjustment();
            let new_volume = (current_volume - adjustment).clamp(0.0, 1.0);

            // Set the new volume
            if let Err(e) = audio::set_session_volume(&session, new_volume) {
                println!("Error setting volume: {:?}", e);
            }
        }
        Err(e) => println!("Error getting focused window session: {:?}", e)
    }
}

pub fn set_acceleration_parameters(max: f32, min: f32, decay: f32) {
    if let Ok(mut state) = VOLUME_STATE.lock() {
        state.max_acceleration = max;
        state.min_acceleration = min;
        state.decay_rate = decay;
    }
}

pub fn set_base_increment(increment: f32) {
    if let Ok(mut state) = VOLUME_STATE.lock() {
        state.base_increment = increment;
    }
}

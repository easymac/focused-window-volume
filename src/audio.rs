use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::CLSCTX_ALL;
use windows::core::ComInterface;
use crate::focus;

pub fn enumerate_audio_sessions() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Initialize COM library
        windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_APARTMENTTHREADED)?;

        // Create a multimedia device enumerator
        let enumerator: IMMDeviceEnumerator = windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, windows::Win32::System::Com::CLSCTX_ALL)?;

        // Get the default audio endpoint
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // Activate the audio session manager
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, Some(std::ptr::null_mut()))?;

        // Get the audio session enumerator
        let session_enumerator = session_manager.GetSessionEnumerator()?;

        // Get the number of audio sessions
        let count = session_enumerator.GetCount()?;

        println!("Number of audio sessions: {}", count);

        for i in 0..count {
            // Get the basic session control
            let session_control = session_enumerator.GetSession(i)?;
            
            // Use the cast method to convert to IAudioSessionControl2
            let session_control2: IAudioSessionControl2 = session_control.cast()?;
            
            // Now we can access methods specific to IAudioSessionControl2
            let display_name = session_control2.GetDisplayName()?;
            let session_pid = session_control2.GetProcessId()?;
            let volume = get_session_volume(&session_control2)?;
            let (pid, process_path) = focus::get_focused_window_details()?;
            
            println!("Session {}: Display Name: {:?}, PID: {}, Volume: {}, Process Path: {}", i, display_name, session_pid, volume, process_path);
        }

        Ok(())
    }
}

pub fn get_session_by_process_path(process_path: &str) -> Result<IAudioSessionControl2, Box<dyn std::error::Error>> {
    println!("Getting session by process path: {}", process_path);
    unsafe {
        // Initialize COM library
        windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_APARTMENTTHREADED)?;

        // Create a multimedia device enumerator
        let enumerator: IMMDeviceEnumerator = windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, windows::Win32::System::Com::CLSCTX_ALL)?;

        // Get the default audio endpoint
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // Activate the audio session manager
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, Some(std::ptr::null_mut()))?;

        // Get the audio session enumerator
        let session_enumerator = session_manager.GetSessionEnumerator()?;

        // Get the number of audio sessions
        let count = session_enumerator.GetCount()?;

        for i in 0..count {
            // Get the basic session control
            let session_control = session_enumerator.GetSession(i)?;
            
            // Use the cast method to convert to IAudioSessionControl2
            let session_control2: IAudioSessionControl2 = session_control.cast()?;
            
            // Get the process ID of the session
            let session_pid = session_control2.GetProcessId()?;
            
            if session_pid == 0 {
                // Skip the system session
                // (in the future we can handle this)
                continue;
            }

            // Get the process path of the session
            let session_path = focus::get_process_path(session_pid)?;
            if session_path == process_path {
                return Ok(session_control2);
            }
        }

        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Session not found")))
    }
}   
pub fn get_session_by_pid(pid: u32) -> Result<IAudioSessionControl2, Box<dyn std::error::Error>> {
    unsafe {
        // Initialize COM library
        windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_APARTMENTTHREADED)?;

        // Create a multimedia device enumerator
        let enumerator: IMMDeviceEnumerator = windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, windows::Win32::System::Com::CLSCTX_ALL)?;

        // Get the default audio endpoint
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // Activate the audio session manager
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, Some(std::ptr::null_mut()))?;

        // Get the audio session enumerator
        let session_enumerator = session_manager.GetSessionEnumerator()?;

        // Get the number of audio sessions
        let count = session_enumerator.GetCount()?;
        println!("Number of audio sessions: {}", count);
        
        for i in 0..count {
            // Get the basic session control
            let session_control = session_enumerator.GetSession(i)?;
            
            // Use the cast method to convert to IAudioSessionControl2
            let session_control2: IAudioSessionControl2 = session_control.cast()?;
            
            // Get the process ID of the session
            let session_pid = session_control2.GetProcessId()?;
            
            if pid == session_pid {
                return Ok(session_control2);
            }
        }
        
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Session not found")))
    }
}




/// Gets the master volume level for a specific audio session
/// 
/// Returns a float between 0.0 (muted) and 1.0 (full volume)
pub fn get_session_volume(session_control: &IAudioSessionControl2) -> Result<f32, Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;
        
        // Get the current master volume level directly from the method return
        let volume_level = simple_audio_volume.GetMasterVolume()?;
        
        Ok(volume_level)
    }
}

pub fn set_session_volume(session_control: &IAudioSessionControl2, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;

        // Set the master volume level
        simple_audio_volume.SetMasterVolume(volume, std::ptr::null())?;

        Ok(())
    }
}

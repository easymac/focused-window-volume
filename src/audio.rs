use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::CLSCTX_ALL;
use windows::core::ComInterface;
use crate::focus;


pub fn get_session_by_process_path(process_path: &str) -> Result<IAudioSessionControl2, Box<dyn std::error::Error>> {
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


/// Gets the master volume level for a specific audio session
/// 
/// Returns a float between 0.0 (muted) and 1.0 (full volume)
fn get_session_volume(session_control: &IAudioSessionControl2) -> Result<f32, Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;
        
        // Get the current master volume level directly from the method return
        let volume_level = simple_audio_volume.GetMasterVolume()?;
        
        Ok(volume_level)
    }
}

fn set_session_volume(session_control: &IAudioSessionControl2, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;

        // Set the master volume level
        simple_audio_volume.SetMasterVolume(volume, std::ptr::null())?;

        Ok(())
    }
}


pub fn mute_session(session_control: &IAudioSessionControl2) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;

        // Mute the session
        simple_audio_volume.SetMute(true, std::ptr::null())?;

        Ok(())
    }
}

pub fn unmute_session(session_control: &IAudioSessionControl2) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;

        // Unmute the session
        simple_audio_volume.SetMute(false, std::ptr::null())?;

        Ok(())
    }
}

pub fn toggle_session_mute(session_control: &IAudioSessionControl2) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get the simple audio volume interface from the session control
        let simple_audio_volume: ISimpleAudioVolume = session_control.cast()?;

        // Toggle the mute state
        let current_mute = simple_audio_volume.GetMute()?;
        simple_audio_volume.SetMute(!current_mute, std::ptr::null())?;

        Ok(())
    }
}



pub fn increment_session_volume(session_control: &IAudioSessionControl2, increment: f32) -> Result<f32, Box<dyn std::error::Error>> {
    let current_volume = get_session_volume(session_control)?;
    let new_volume = current_volume + increment;
    let new_volume = if new_volume > 1.0 { 1.0 } else { new_volume };

    set_session_volume(session_control, new_volume)?;
    Ok(new_volume)
}

pub fn decrement_session_volume(session_control: &IAudioSessionControl2, decrement: f32) -> Result<f32, Box<dyn std::error::Error>> {
    let current_volume = get_session_volume(session_control)?;
    let new_volume = current_volume - decrement;
    let new_volume = if new_volume < 0.0 { 0.0 } else { new_volume };

    set_session_volume(session_control, new_volume)?;
    Ok(new_volume)
}

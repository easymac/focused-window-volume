use std::sync::mpsc;
use std::mem;
use std::ffi::c_void;
use std::ptr::null;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

const WM_APP_NOTIFY: u32 = WM_APP + 1;
const IDM_PRINT: u32 = 1000;
const IDM_EXIT: u32 = 1001;

pub enum TrayEvent {
    Print,
    Quit,
}

pub struct Tray {
    hwnd: HWND,
    rx: mpsc::Receiver<TrayEvent>,
}

impl Tray {
    pub fn new(app_name: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        
        // Clone tx and create a boxed copy to be stored in the window
        let boxed_tx = Box::new(tx.clone());
        let tx_raw = Box::into_raw(boxed_tx);
        
        // Create a hidden window to receive messages
        let hwnd = unsafe {
            let instance = GetModuleHandleW(None)?;
            let class_name = w!("FocusedWindowVolumeTrayClass");
            
            let wc = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_VREDRAW | CS_HREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance.into(),
                hIcon: HICON(0),
                hCursor: HCURSOR(0),
                hbrBackground: HBRUSH(0),
                lpszMenuName: PCWSTR(null()),
                lpszClassName: class_name,
                hIconSm: HICON(0),
            };
            
            let atom = RegisterClassExW(&wc);
            if atom == 0 {
                return Err(Error::from_win32().into());
            }
            
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                w!("Focused Window Volume"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                Some(tx_raw as *const c_void),
            );
            
            if hwnd.0 == 0 {
                return Err(Error::from_win32().into());
            }
            
            // Add the system tray icon
            let mut nid = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NOTIFY_ICON_DATA_FLAGS(0x1 | 0x4 | 0x2),
                uCallbackMessage: WM_APP_NOTIFY,
                hIcon: LoadIconW(instance, w!("speaker"))?,
                szTip: [0; 128],
                dwState: NOTIFY_ICON_STATE(0),
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: Default::default(),
                szInfoTitle: [0; 64],
                dwInfoFlags: NOTIFY_ICON_INFOTIP_FLAGS(0),
                guidItem: GUID::default(),
                hBalloonIcon: HICON(0),
            };
            
            // Copy the app name to the tooltip
            let app_name_wide: Vec<u16> = app_name.encode_utf16().chain(Some(0)).collect();
            let max_len = nid.szTip.len().min(app_name_wide.len());
            nid.szTip[..max_len].copy_from_slice(&app_name_wide[..max_len]);
            
            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            if !result.as_bool() {
                return Err(Error::from_win32().into());
            }
            
            hwnd
        };
        
        Ok(Self {
            hwnd,
            rx,
        })
    }
    
    pub fn run(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Start the message loop
            let mut msg = MSG::default();
            
            loop {
                // Check for Windows messages
                if PeekMessageW(&mut msg, HWND(0), 0, 0, PM_REMOVE).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    
                    if msg.message == WM_QUIT {
                        break;
                    }
                }
                
                // Check for events from the channel
                match self.rx.try_recv() {
                    Ok(TrayEvent::Print) => {
                        println!("Tray item clicked!");
                    },
                    Ok(TrayEvent::Quit) => {
                        println!("Quitting application...");
                        break;
                    },
                    Err(mpsc::TryRecvError::Empty) => {
                        // No message available, sleep a bit to avoid busy waiting
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    },
                    Err(mpsc::TryRecvError::Disconnected) => {
                        eprintln!("Channel disconnected");
                        break;
                    }
                }
            }
            
            // Clean up the tray icon
            let nid = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                uFlags: NOTIFY_ICON_DATA_FLAGS(0),
                uCallbackMessage: 0,
                hIcon: HICON(0),
                szTip: [0; 128],
                dwState: NOTIFY_ICON_STATE(0),
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: Default::default(),
                szInfoTitle: [0; 64],
                dwInfoFlags: NOTIFY_ICON_INFOTIP_FLAGS(0),
                guidItem: GUID::default(),
                hBalloonIcon: HICON(0),
            };
            
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
        }
        
        Ok(())
    }
}

impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            // Clean up the tray icon
            let nid = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: 1,
                uFlags: NOTIFY_ICON_DATA_FLAGS(0),
                uCallbackMessage: 0,
                hIcon: HICON(0),
                szTip: [0; 128],
                dwState: NOTIFY_ICON_STATE(0),
                dwStateMask: 0,
                szInfo: [0; 256],
                Anonymous: Default::default(),
                szInfoTitle: [0; 64],
                dwInfoFlags: NOTIFY_ICON_INFOTIP_FLAGS(0),
                guidItem: GUID::default(),
                hBalloonIcon: HICON(0),
            };
            
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(self.hwnd);
        }
    }
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        // Get the channel sender from the window's user data
        let tx_ptr = if msg == WM_CREATE {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let tx_ptr = (*create_struct).lpCreateParams;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, tx_ptr as isize);
            tx_ptr as *const mpsc::Sender<TrayEvent>
        } else {
            let user_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            user_data as *const mpsc::Sender<TrayEvent>
        };
        
        match msg {
            WM_APP_NOTIFY => {
                // Tray icon was clicked
                match lparam.0 as u32 {
                    WM_CONTEXTMENU | WM_RBUTTONUP => {
                        // Show context menu
                        let hmenu = CreatePopupMenu().unwrap();
                        AppendMenuW(hmenu, MENU_ITEM_FLAGS(0), IDM_PRINT as usize, w!("Print Message")).unwrap();
                        AppendMenuW(hmenu, MENU_ITEM_FLAGS(0), IDM_EXIT as usize, w!("Quit")).unwrap();
                        
                        // Get cursor position
                        let mut pt = POINT::default();
                        GetCursorPos(&mut pt).unwrap();
                        
                        // Show menu and track click
                        SetForegroundWindow(hwnd);
                        TrackPopupMenu(
                            hmenu,
                            TPM_LEFTALIGN | TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            0,
                            hwnd,
                            None,
                        ).unwrap();
                        PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0)).unwrap();
                        
                        DestroyMenu(hmenu).unwrap();
                        LRESULT(0)
                    },
                    _ => LRESULT(0),
                }
            },
            WM_COMMAND => {
                let wmid = LOWORD(wparam.0 as u32);
                match wmid {
                    IDM_PRINT => {
                        if !tx_ptr.is_null() {
                            let tx = &*tx_ptr;
                            if let Err(e) = tx.send(TrayEvent::Print) {
                                eprintln!("Failed to send print event: {}", e);
                            }
                        }
                        LRESULT(0)
                    },
                    IDM_EXIT => {
                        if !tx_ptr.is_null() {
                            let tx = &*tx_ptr;
                            if let Err(e) = tx.send(TrayEvent::Quit) {
                                eprintln!("Failed to send quit event: {}", e);
                            }
                        }
                        LRESULT(0)
                    },
                    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            },
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            },
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// Helper function to extract the low-order word from a value
#[inline]
fn LOWORD(value: u32) -> u32 {
    value & 0xFFFF
} 
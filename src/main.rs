// This utility is designed to work around Houdini Indie's inability to handle OneDrive path related errors
// without constantly blocking input with a console window. This is aimed at the Steam version of Houdini Indie.
// A red status bar message will still be visible within Houdini.
// Please read the source, proceed with caution, and use at your own risk.
// See the LICENSE file for details.

#![windows_subsystem = "windows"]

use std::{os::windows::process::CommandExt, process::Command, thread, time::Duration};

const CREATE_NO_WINDOW: u32 = 0x0800_0000; // CREATE_NO_WINDOW
const DETACHED_PROCESS: u32 = 0x0000_0008; // DETACHED_PROCESS  
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200; // CREATE_NEW_PROCESS_GROUP

/// Hide any console window that might appear after launch.
unsafe fn hide_console_windows() {
    use std::ffi::CString;
    use winapi::um::winuser::{FindWindowA, SW_HIDE, ShowWindow};

    let window_name = CString::new("Houdini Console").unwrap();
    let window = unsafe { FindWindowA(std::ptr::null(), window_name.as_ptr()) };
    if !window.is_null() {
        unsafe { ShowWindow(window, SW_HIDE) };
    }
}

fn is_houdini_running() -> bool {
    use winapi::shared::minwindef::{BOOL, LPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowTextA};

    static mut HOUDINI_FOUND: bool = false;

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        let mut buffer = [0i8; 256];
        let len = unsafe { GetWindowTextA(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
        if len > 0 {
            let title = unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }
                .to_string_lossy()
                .to_lowercase();
            // Look for actual Houdini application windows (not just any random window with "houdini")
            if title.contains("houdini") && title.contains("indie") {
                unsafe { HOUDINI_FOUND = true };
                return 0; // Stop enumeration.
            }
        }
        1 // Continue enumeration.
    }

    unsafe { HOUDINI_FOUND = false };
    unsafe { EnumWindows(Some(enum_window_proc), 0) };
    unsafe { HOUDINI_FOUND }
}

fn main() {
    // Hard‐coded path to Steam's Houdini Indie launcher. Adjust if yours is in a different location:
    let houdini_path =
        r"C:\Program Files (x86)\Steam\steamapps\common\Houdini Indie\bin\hindie.steam.exe";

    // Build a Command to launch: hindie.steam.exe
    let mut cmd = Command::new(houdini_path);
    cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

    match cmd.spawn() {
        Ok(_child) => {
            // Give Houdini a moment to start up through Steam.
            for _ in 0..3000 {
                thread::sleep(Duration::from_millis(20));
                unsafe {
                    hide_console_windows();
                }
                if is_houdini_running() {
                    break;
                }
            }

            // Continuously monitor and hide console windows while any Houdini process is running.
            loop {
                unsafe {
                    hide_console_windows();
                }

                // Check if any Houdini processes are still running.
                if !is_houdini_running() {
                    break; // Exit when no Houdini processes are found.
                }

                // Check for console windows continuously.
                thread::sleep(Duration::from_millis(0));
            }
        }
        Err(err) => {
            // If for some reason Houdini fails to start, write an error message to a simple pop-up.
            use std::ffi::CString;
            use winapi::um::winuser::{MB_ICONERROR, MB_OK, MessageBoxA};

            let msg = CString::new(format!("Failed to launch Houdini Indie:\n\n{}", err)).unwrap();
            let title = CString::new("Error").unwrap();

            unsafe {
                MessageBoxA(
                    std::ptr::null_mut(),
                    msg.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
    }
}

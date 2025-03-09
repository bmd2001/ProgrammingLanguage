#[cfg(target_os = "linux")]
pub const TARGET_OS: OS = OS::Linux;

#[cfg(target_os = "macos")]
pub const TARGET_OS: OS = OS::MacOS;

#[cfg(target_os = "windows")]
pub const TARGET_OS: OS = OS::Windows;

#[allow(dead_code)] // Only one arch will be used, so it's an expected behaviour
#[derive(Debug)]
pub enum OS {
    Linux,
    MacOS,
    Windows
}
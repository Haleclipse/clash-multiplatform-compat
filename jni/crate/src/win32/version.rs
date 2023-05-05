use std::mem::size_of;
use windows::Win32::{
    Foundation::TRUE,
    System::SystemInformation::{GetVersionExA, OSVERSIONINFOA},
};

pub fn is_supported() -> bool {
    let mut version = OSVERSIONINFOA::default();

    version.dwOSVersionInfoSize = size_of::<OSVERSIONINFOA>() as u32;

    unsafe { GetVersionExA(&mut version) == TRUE && version.dwMajorVersion >= 10 && version.dwBuildNumber >= 17063 }
}

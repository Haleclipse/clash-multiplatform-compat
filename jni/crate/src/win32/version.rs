use std::mem::size_of;
use windows::Win32::{
    Foundation::TRUE,
    System::SystemInformation::{GetVersionExW, OSVERSIONINFOW},
};

pub fn is_supported() -> bool {
    let mut version = OSVERSIONINFOW::default();

    version.dwOSVersionInfoSize = size_of::<OSVERSIONINFOW>() as u32;

    unsafe { GetVersionExW(&mut version) == TRUE && version.dwMajorVersion >= 10 && version.dwBuildNumber >= 17063 }
}

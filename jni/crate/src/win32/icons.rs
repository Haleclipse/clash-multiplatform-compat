use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::win32::{error::Error, strings::Win32StringIntoExt};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::WIN32_ERROR,
        UI::{
            Shell::*,
            WindowsAndMessaging::{GetSystemMetrics, LoadImageW, HICON, IMAGE_ICON, LR_LOADFROMFILE, SM_CXICON, SM_CYICON},
        },
    },
};

pub fn get_icons_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_path: String = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, KNOWN_FOLDER_FLAG(0), None)?.to_string()? };

    Ok(Path::new(&local_path).join("Icons").join(name).with_extension("ico"))
}

pub fn load_icon(name: &str) -> Result<HICON, Box<dyn std::error::Error>> {
    let icon = unsafe {
        let name_utf16 = get_icons_path(name)?.to_str().unwrap().to_win32_utf16();

        LoadImageW(
            None,
            PCWSTR::from_raw(name_utf16.as_ptr()),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
            LR_LOADFROMFILE,
        )
        .map_err(|e| Error::new("LoadImageW", WIN32_ERROR(e.code().0 as u32)))?
    };

    Ok(HICON(icon.0))
}

pub fn install_icon(name: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let icon_path = get_icons_path(name)?;

    fs::create_dir_all(icon_path.parent().unwrap())?;

    fs::write(icon_path, data)?;

    Ok(())
}

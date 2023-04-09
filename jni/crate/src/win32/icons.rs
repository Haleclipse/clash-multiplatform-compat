use std::{
    ffi::CString,
    fs,
    path::{Path, PathBuf},
};

use windows::{
    core::PCSTR,
    Win32::UI::{
        Shell::*,
        WindowsAndMessaging::{GetSystemMetrics, LoadImageA, HICON, IMAGE_ICON, LR_LOADFROMFILE, SM_CXICON, SM_CYICON},
    },
};

pub fn get_icons_path(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_path: String = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, KNOWN_FOLDER_FLAG(0), None)?.to_string()? };

    Ok(Path::new(&local_path).join("Icons").join(name).with_extension("ico"))
}

pub fn load_icon(name: &str) -> Result<HICON, Box<dyn std::error::Error>> {
    let icon = unsafe {
        let icon_path = CString::new(get_icons_path(name)?.to_str().unwrap())?;

        LoadImageA(
            None,
            PCSTR(icon_path.as_ptr().cast()),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
            LR_LOADFROMFILE,
        )?
    };

    Ok(HICON(icon.0))
}

pub fn install_icon(name: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let icon_path = get_icons_path(name)?;

    fs::create_dir_all(icon_path.parent().unwrap())?;

    fs::write(icon_path, data)?;

    Ok(())
}

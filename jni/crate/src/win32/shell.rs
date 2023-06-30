use std::{
    error::Error,
    ffi::OsString,
    iter::once,
    mem::size_of,
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    ptr::null_mut,
    sync::Once,
};

use windows::{
    core::{ComInterface, PCWSTR, PWSTR},
    w,
    Win32::{
        Foundation::{GetLastError, ERROR_SUCCESS, HWND, MAX_PATH, TRUE},
        Storage::EnhancedStorage::PKEY_AppUserModel_ID,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, IPersistFile, StructuredStorage::PropVariantClear, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED, STGM_READ,
            },
            Registry::{
                RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
                KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, REG_VALUE_TYPE,
            },
        },
        UI::{
            Controls::Dialogs::{
                GetOpenFileNameW, GetSaveFileNameW, OFN_FILEMUSTEXIST, OFN_OVERWRITEPROMPT, OPENFILENAMEW, OPEN_FILENAME_FLAGS,
            },
            Shell::{
                PropertiesSystem::{IPropertyStore, InitPropVariantFromStringAsVector, PropVariantToString},
                *,
            },
            WindowsAndMessaging::{SW_HIDE, SW_SHOW},
        },
    },
};

use crate::{
    common::shell::FileFilter,
    utils::scoped::Scoped,
    win32,
    win32::{
        icons,
        icons::get_icons_path,
        strings::{join_arguments, Win32StringFromExt, Win32StringIntoExt},
    },
};

fn to_win32_filters(filters: &[FileFilter]) -> Vec<u16> {
    let mut ret = Vec::with_capacity(64);

    for filter in filters {
        ret.extend(filter.label.encode_utf16());
        ret.push(0);

        for extension in &filter.extensions {
            let expr = format!("*.{extension}");

            ret.extend(expr.encode_utf16());
            ret.extend(";".encode_utf16());
        }
        ret.push(0);
    }
    ret.push(0);

    ret
}

fn build_open_file_name(
    window: i64,
    title: &[u16],
    filters: &[u16],
    flags: OPEN_FILENAME_FLAGS,
    ret: &mut [u16],
) -> OPENFILENAMEW {
    let mut open_file_name = OPENFILENAMEW::default();
    open_file_name.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    open_file_name.hwndOwner = HWND(window as isize);
    open_file_name.lpstrTitle = PCWSTR::from_raw(title.as_ptr());
    open_file_name.lpstrFilter = PCWSTR::from_raw(filters.as_ptr());
    open_file_name.lpstrFile = PWSTR::from_raw(ret.as_mut_ptr());
    open_file_name.nMaxFile = (ret.len() - 1) as u32;
    open_file_name.Flags = flags;

    let initial_dir = std::env::var("USERPROFILE").map(|s| s.to_win32_utf16());
    if let Ok(dir) = &initial_dir {
        open_file_name.lpstrInitialDir = PCWSTR::from_raw(dir.as_ptr());
    }

    open_file_name
}

pub fn run_pick_file(window: i64, title: &str, filters: &[FileFilter]) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let title = title.to_win32_utf16();
    let filters = to_win32_filters(filters);
    let mut ret: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let mut open_file_name = build_open_file_name(window, &title, &filters, OFN_FILEMUSTEXIST, &mut ret);

    unsafe {
        if GetOpenFileNameW(&mut open_file_name as *mut OPENFILENAMEW) == TRUE {
            Ok(Some(PathBuf::from(String::from_win32_utf16(&ret)?)))
        } else {
            if GetLastError() != ERROR_SUCCESS {
                Err(Box::new(win32::error::Error::with_current("GetOpenFileNameW")))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn run_save_file(
    window: i64,
    file_name: &str,
    title: &str,
    filters: &[FileFilter],
) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let title = title.to_win32_utf16();
    let filters = to_win32_filters(filters);

    let mut ret: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    file_name
        .encode_utf16()
        .enumerate()
        .take(ret.len() - 1)
        .for_each(|(idx, wc)| ret[idx] = wc);

    let mut open_file_name = build_open_file_name(window, &title, &filters, OFN_OVERWRITEPROMPT, &mut ret);

    unsafe {
        if GetSaveFileNameW(&mut open_file_name) == TRUE {
            Ok(Some(PathBuf::from(String::from_win32_utf16(&ret)?)))
        } else {
            if GetLastError() != ERROR_SUCCESS {
                Err(Box::new(win32::error::Error::with_current("GetSaveFileNameW")))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn run_launch_file(window: i64, file: &str) -> Result<(), Box<dyn Error>> {
    let file = file.to_win32_utf16();

    let ret = unsafe {
        ShellExecuteW(
            HWND(window as isize),
            w!("open"),
            PCWSTR::from_raw(file.as_ptr()),
            None,
            None,
            SW_SHOW,
        )
    };

    if ret.0 > 32 {
        Ok(())
    } else {
        Err(Box::new(win32::error::Error::with_current("ShellExecuteW")))
    }
}

pub fn install_icon(name: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    icons::install_icon(name, data)
}

static CO_INITIALIZE_ONCE: Once = Once::new();

fn initialize_com() {
    CO_INITIALIZE_ONCE.call_once(|| unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();
    })
}

fn get_shortcut_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let programs_dir = unsafe { SHGetKnownFolderPath(&FOLDERID_Programs, KNOWN_FOLDER_FLAG::default(), None)?.to_string()? };

    Ok(Path::new(&programs_dir).join(name).with_extension("lnk"))
}

fn valid_shortcut(app_id: &str, name: &str, icon: &str, executable: &str, arguments: &[String]) -> Result<(), Box<dyn Error>> {
    let link_path = get_shortcut_path(name)?;
    if !link_path.exists() {
        return Err("file not found".into());
    }

    unsafe {
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        let persist_file: IPersistFile = shell_link.cast()?;
        let link_path = link_path.to_string_lossy().encode_utf16().chain(once(0)).collect::<Vec<_>>();
        persist_file.Load(PCWSTR(link_path.as_ptr()), STGM_READ)?;

        let properties_store: IPropertyStore = shell_link.cast()?;

        let mut buffer = [0u16; MAX_PATH as usize];

        let app_id_prop = properties_store.GetValue(&PKEY_AppUserModel_ID)?;
        PropVariantToString(&app_id_prop, &mut buffer)?;

        let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
        if String::from_utf16(&buffer[..end])? != app_id {
            return Err("application id not match".into());
        }

        let mut buffer = [0u16; MAX_PATH as usize];

        let mut icon_index = -1;
        shell_link.GetIconLocation(&mut buffer, &mut icon_index)?;
        if OsString::from_wide(&buffer).to_string_lossy() != get_icons_path(icon)?.to_string_lossy() || icon_index != 0 {
            return Err("icon not match".into());
        }

        shell_link.GetPath(&mut buffer, null_mut(), SLGP_RAWPATH.0 as u32)?;
        if OsString::from_wide(&buffer).to_string_lossy() != executable {
            return Err("executable not match".into());
        }

        shell_link.GetArguments(&mut buffer)?;
        if OsString::from_wide(&buffer).to_string_lossy() != join_arguments(arguments) {
            return Err("arguments not match".into());
        }
    }

    Ok(())
}

pub fn install_shortcut(
    app_id: &str,
    name: &str,
    icon: &str,
    executable: &str,
    arguments: &[String],
) -> Result<(), Box<dyn Error>> {
    initialize_com();

    if let Ok(_) = valid_shortcut(app_id, name, icon, executable, arguments) {
        return Ok(());
    }

    _ = uninstall_shortcut(app_id, name);

    unsafe {
        let link_path = get_shortcut_path(name)?;
        let icon = get_icons_path(icon)?.to_str().unwrap().to_win32_utf16();
        let executable = executable.to_win32_utf16();
        let arguments = join_arguments(arguments).to_win32_utf16();
        let working_dir = std::env::current_dir()?.to_str().unwrap().to_win32_utf16();

        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell_link.SetPath(PCWSTR::from_raw(executable.as_ptr()))?;
        shell_link.SetArguments(PCWSTR::from_raw(arguments.as_ptr()))?;
        shell_link.SetWorkingDirectory(PCWSTR::from_raw(working_dir.as_ptr()))?;
        shell_link.SetIconLocation(PCWSTR::from_raw(icon.as_ptr()), 0)?;
        shell_link.SetShowCmd(SW_HIDE)?;

        let properties: IPropertyStore = shell_link.cast()?;

        let app_id = app_id.to_win32_utf16();
        let mut app_id = InitPropVariantFromStringAsVector(PCWSTR::from_raw(app_id.as_ptr()))?;

        let set_result = properties.SetValue(&PKEY_AppUserModel_ID, &app_id);

        PropVariantClear(&mut app_id).ok();

        set_result?;

        properties.Commit()?;

        let link_path = link_path.to_str().unwrap().to_win32_utf16();

        shell_link
            .cast::<IPersistFile>()?
            .Save(PCWSTR::from_raw(link_path.as_ptr()), TRUE)?;
    }

    Ok(())
}

pub fn uninstall_shortcut(_: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let link_path = get_shortcut_path(name)?;

    std::fs::remove_file(link_path)?;

    Ok(())
}

unsafe fn open_run_key(mode: REG_SAM_FLAGS) -> Result<Scoped<HKEY, fn(&HKEY)>, Box<dyn Error>> {
    let mut key: Scoped<HKEY, fn(&HKEY)> = Scoped::new(Default::default(), |key| {
        RegCloseKey(*key);
    });

    RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"),
        0,
        mode,
        &mut *key,
    )
    .ok()?;

    Ok(key)
}

pub fn is_run_on_boot_existed(app_id: &str) -> bool {
    unsafe {
        let key: Scoped<HKEY, _> = if let Ok(key) = open_run_key(KEY_QUERY_VALUE) {
            key
        } else {
            return false;
        };

        let app_id = app_id.to_win32_utf16();

        let mut sub_key_type: REG_VALUE_TYPE = Default::default();
        RegQueryValueExW(
            *key,
            PCWSTR::from_raw(app_id.as_ptr()),
            None,
            Some(&mut sub_key_type),
            None,
            None,
        )
        .is_ok()
    }
}

pub fn set_run_on_boot(app_id: &str, executable: &str, arguments: &[String]) -> Result<(), Box<dyn Error>> {
    unsafe {
        let key: Scoped<HKEY, _> = open_run_key(KEY_SET_VALUE)?;

        let app_id = app_id.to_win32_utf16();
        let command_line = format!("\"{}\" {}", executable, join_arguments(arguments)).to_win32_utf16();
        RegSetValueExW(
            *key,
            PCWSTR::from_raw(app_id.as_ptr()),
            0,
            REG_SZ,
            Some(std::slice::from_raw_parts(
                command_line.as_ptr() as *const u8,
                command_line.len() * 2,
            )),
        )
        .ok()?;

        Ok(())
    }
}

pub fn remove_run_on_boot(app_id: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        let key = open_run_key(KEY_SET_VALUE | KEY_QUERY_VALUE)?;

        let app_id = app_id.to_win32_utf16();

        RegDeleteValueW(*key, PCWSTR::from_raw(app_id.as_ptr())).ok()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, time::Duration};

    use crate::win32::{
        icons::install_icon,
        notification::send_notification,
        shell::{install_shortcut, uninstall_shortcut},
        testdata::TestData,
    };

    const TEST_APP_ID: &str = "com.github.kr328.clash.compat.CompatLibrary";
    const TEST_APP_NAME: &str = "Clash Compat Library";
    const TEST_APP_ICON_NAME: &str = "clash-multiplatform-compat";
    const TEST_APP_ICON_PATH: &str = "clash-multiplatform.ico";

    #[test]
    pub fn remove_shortcut() -> Result<(), Box<dyn Error>> {
        uninstall_shortcut(TEST_APP_NAME, TEST_APP_ID)?;

        Ok(())
    }

    #[test]
    pub fn create_shortcut() -> Result<(), Box<dyn Error>> {
        let icon = TestData::get(TEST_APP_ICON_PATH).unwrap().data;

        install_icon(TEST_APP_ICON_NAME, &icon)?;

        let executable = std::env::current_exe()?;

        install_shortcut(
            TEST_APP_ID,
            TEST_APP_NAME,
            TEST_APP_ICON_NAME,
            executable.to_str().unwrap(),
            &[],
        )?;

        send_notification(TEST_APP_ID, "Shortcut Installation", "Shortcut installed")?;

        std::thread::sleep(Duration::from_secs(10));

        Ok(())
    }
}

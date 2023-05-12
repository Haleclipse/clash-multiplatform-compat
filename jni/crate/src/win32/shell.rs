use std::{
    error::Error,
    ffi::{CStr, CString},
    iter::once,
    mem::size_of,
    path::{Path, PathBuf},
    ptr::null_mut,
    sync::Once,
};

use cstr::cstr;
use windows::{
    core::{ComInterface, PCSTR, PCWSTR, PSTR},
    Win32::{
        Foundation::{GetLastError, ERROR_SUCCESS, HWND, MAX_PATH, TRUE},
        Storage::EnhancedStorage::PKEY_AppUserModel_ID,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, IPersistFile, StructuredStorage::PropVariantClear, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED, STGM_READ,
            },
            Registry::{
                RegCloseKey, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA, RegSetValueExA, HKEY, HKEY_CURRENT_USER,
                KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SAM_FLAGS, REG_SZ, REG_VALUE_TYPE,
            },
        },
        UI::{
            Controls::Dialogs::{GetOpenFileNameA, OFN_FILEMUSTEXIST, OPENFILENAMEA},
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
        strings::{join_arguments, string_to_os_utf16},
    },
};

pub fn run_pick_file(window: i64, title: &str, filters: &[FileFilter]) -> Result<Option<PathBuf>, Box<dyn Error>> {
    let title = CString::new(title)?;

    let mut joined_filters: Vec<u8> = Vec::with_capacity(64);
    for filter in filters {
        joined_filters.extend_from_slice(filter.label.as_bytes());
        joined_filters.push(b'\0');

        for extension in &filter.extensions {
            let expr = format!("*.{extension}");

            joined_filters.extend_from_slice(expr.as_bytes());
            joined_filters.push(b';');
        }
        joined_filters.push(b'\0');
    }
    joined_filters.push(b'\0');

    let mut ret: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let mut open_file_name = OPENFILENAMEA::default();
    open_file_name.lStructSize = size_of::<OPENFILENAMEA>() as u32;
    open_file_name.hwndOwner = HWND(window as isize);
    open_file_name.lpstrTitle = PCSTR(title.as_ptr().cast());
    open_file_name.lpstrFilter = PCSTR(joined_filters.as_ptr());
    open_file_name.lpstrFile = PSTR(ret.as_mut_ptr());
    open_file_name.nMaxFile = (ret.len() - 1) as u32;
    open_file_name.Flags = OFN_FILEMUSTEXIST;

    let initial_dir = std::env::var("USERPROFILE").map(|s| CString::new(s));
    if let Ok(Ok(dir)) = &initial_dir {
        open_file_name.lpstrInitialDir = PCSTR(dir.as_ptr().cast());
    }

    unsafe {
        if GetOpenFileNameA(&mut open_file_name as *mut OPENFILENAMEA) == TRUE {
            Ok(Some(PathBuf::from(CStr::from_ptr(ret.as_ptr().cast()).to_str()?.to_string())))
        } else {
            if GetLastError() != ERROR_SUCCESS {
                Err(Box::new(win32::error::Error::with_current("GetOpenFileNameA")))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn run_launch_file(window: i64, file: &str) -> Result<(), Box<dyn Error>> {
    let file = CString::new(file)?;

    let ret = unsafe {
        ShellExecuteA(
            HWND(window as isize),
            PCSTR(cstr!("open").as_ptr().cast()),
            PCSTR(file.as_ptr().cast()),
            None,
            None,
            SW_SHOW,
        )
    };

    if ret.0 > 32 {
        Ok(())
    } else {
        Err(Box::new(win32::error::Error::with_current("ShellExecuteA")))
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
        let shell_link: IShellLinkA = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

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

        let mut buffer = [0u8; MAX_PATH as usize];

        let mut icon_index = -1;
        shell_link.GetIconLocation(&mut buffer, &mut icon_index)?;
        if CStr::from_bytes_until_nul(&buffer)?.to_str()? != get_icons_path(icon)?.to_string_lossy() || icon_index != 0 {
            return Err("icon not match".into());
        }

        shell_link.GetPath(&mut buffer, null_mut(), SLGP_RAWPATH.0 as u32)?;
        if CStr::from_bytes_until_nul(&buffer)?.to_str()? != executable {
            return Err("executable not match".into());
        }

        shell_link.GetArguments(&mut buffer)?;
        if CStr::from_bytes_until_nul(&buffer)?.to_str()? != join_arguments(arguments) {
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
        let icon = CString::new(get_icons_path(icon)?.to_str().unwrap())?;
        let executable = CString::new(executable)?;
        let arguments = CString::new(join_arguments(arguments))?;
        let working_dir = CString::new(std::env::current_dir()?.to_str().unwrap())?;

        let shell_link: IShellLinkA = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell_link.SetPath(PCSTR(executable.as_ptr().cast()))?;
        shell_link.SetArguments(PCSTR(arguments.as_ptr().cast()))?;
        shell_link.SetWorkingDirectory(PCSTR(working_dir.as_ptr().cast()))?;
        shell_link.SetIconLocation(PCSTR(icon.as_ptr().cast()), 0)?;
        shell_link.SetShowCmd(SW_HIDE)?;

        let properties: IPropertyStore = shell_link.cast()?;

        let mut app_id = InitPropVariantFromStringAsVector(PCWSTR(string_to_os_utf16(app_id).as_ptr()))?;

        let set_result = properties.SetValue(&PKEY_AppUserModel_ID, &app_id);

        PropVariantClear(&mut app_id).ok();

        set_result?;

        properties.Commit()?;

        shell_link
            .cast::<IPersistFile>()?
            .Save(PCWSTR(string_to_os_utf16(link_path.to_str().unwrap()).as_ptr()), TRUE)?;
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

    RegOpenKeyExA(
        HKEY_CURRENT_USER,
        PCSTR(cstr!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run").as_ptr().cast()),
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

        let app_id = if let Ok(id) = CString::new(app_id) {
            id
        } else {
            return false;
        };

        let mut sub_key_type: REG_VALUE_TYPE = Default::default();
        RegQueryValueExA(*key, PCSTR(app_id.as_ptr().cast()), None, Some(&mut sub_key_type), None, None).is_ok()
    }
}

pub fn set_run_on_boot(app_id: &str, executable: &str, arguments: &[String]) -> Result<(), Box<dyn Error>> {
    unsafe {
        let key: Scoped<HKEY, _> = open_run_key(KEY_SET_VALUE)?;

        let app_id = CString::new(app_id)?;
        let command_line = format!("\"{}\" {}", executable, join_arguments(arguments)).into_bytes();
        RegSetValueExA(*key, PCSTR(app_id.as_ptr().cast()), 0, REG_SZ, Some(&command_line)).ok()?;

        Ok(())
    }
}

pub fn remove_run_on_boot(app_id: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        let key = open_run_key(KEY_SET_VALUE | KEY_QUERY_VALUE)?;

        let app_id = CString::new(app_id)?;

        RegDeleteValueA(*key, PCSTR(app_id.as_ptr().cast())).ok()?;

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

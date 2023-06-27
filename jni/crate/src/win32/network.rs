use crate::{common::network::SystemProxyConfig, win32::strings::Win32StringIntoExt};
use std::{error::Error, ffi::c_void, io, mem::size_of, ptr::null};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::FALSE,
        Networking::WinInet::{
            InternetSetOptionW, INTERNET_OPTION_PER_CONNECTION_OPTION, INTERNET_PER_CONN, INTERNET_PER_CONN_FLAGS,
            INTERNET_PER_CONN_FLAGS_UI, INTERNET_PER_CONN_OPTIONW, INTERNET_PER_CONN_OPTIONW_0, INTERNET_PER_CONN_OPTION_LISTW,
            INTERNET_PER_CONN_PROXY_BYPASS, INTERNET_PER_CONN_PROXY_SERVER, PROXY_TYPE_DIRECT, PROXY_TYPE_PROXY,
        },
    },
};

pub fn set_system_proxy(enabled: bool, cfg: &SystemProxyConfig) -> Result<(), Box<dyn Error>> {
    unsafe {
        let flags: u32;
        let address: Option<Vec<u16>>;
        let bypass: Option<Vec<u16>>;

        if enabled {
            flags = PROXY_TYPE_DIRECT | PROXY_TYPE_PROXY;
            address = Some(cfg.address.to_win32_utf16());
            bypass = Some(cfg.excludes.join(";").to_win32_utf16());
        } else {
            flags = PROXY_TYPE_DIRECT;
            address = None;
            bypass = None;
        }

        fn to_pwstr_or_null(s: &Option<Vec<u16>>) -> PWSTR {
            PWSTR::from_raw(s.as_ref().map(|a| a.as_ptr()).unwrap_or(null()).cast_mut().cast())
        }

        let mut options = [
            INTERNET_PER_CONN_OPTIONW {
                dwOption: INTERNET_PER_CONN_FLAGS,
                Value: INTERNET_PER_CONN_OPTIONW_0 { dwValue: flags },
            },
            INTERNET_PER_CONN_OPTIONW {
                dwOption: INTERNET_PER_CONN_PROXY_SERVER,
                Value: INTERNET_PER_CONN_OPTIONW_0 {
                    pszValue: to_pwstr_or_null(&address),
                },
            },
            INTERNET_PER_CONN_OPTIONW {
                dwOption: INTERNET_PER_CONN_PROXY_BYPASS,
                Value: INTERNET_PER_CONN_OPTIONW_0 {
                    pszValue: to_pwstr_or_null(&bypass),
                },
            },
            INTERNET_PER_CONN_OPTIONW {
                dwOption: INTERNET_PER_CONN(INTERNET_PER_CONN_FLAGS_UI),
                Value: INTERNET_PER_CONN_OPTIONW_0 { dwValue: flags },
            },
        ];

        let options = INTERNET_PER_CONN_OPTION_LISTW {
            dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            pszConnection: PWSTR::null(),
            dwOptionCount: options.len() as u32,
            dwOptionError: 0,
            pOptions: (&mut options).as_mut_ptr(),
        };

        if InternetSetOptionW(
            None,
            INTERNET_OPTION_PER_CONNECTION_OPTION,
            Some((&options as *const INTERNET_PER_CONN_OPTION_LISTW).cast::<c_void>()),
            options.dwSize,
        ) == FALSE
        {
            return Err(io::Error::last_os_error().into());
        }
    }

    Ok(())
}

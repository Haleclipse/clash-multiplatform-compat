use crate::common::network::SystemProxyConfig;
use std::{
    error::Error,
    ffi::{c_void, CString},
    io,
    mem::size_of,
    ptr::null,
};
use windows::{
    core::PSTR,
    Win32::{
        Foundation::FALSE,
        Networking::WinInet::{
            InternetSetOptionA, INTERNET_OPTION_PER_CONNECTION_OPTION, INTERNET_PER_CONN, INTERNET_PER_CONN_FLAGS,
            INTERNET_PER_CONN_FLAGS_UI, INTERNET_PER_CONN_OPTIONA, INTERNET_PER_CONN_OPTIONA_0, INTERNET_PER_CONN_OPTION_LISTA,
            INTERNET_PER_CONN_PROXY_BYPASS, INTERNET_PER_CONN_PROXY_SERVER, PROXY_TYPE_DIRECT, PROXY_TYPE_PROXY,
        },
    },
};

pub fn set_system_proxy(enabled: bool, cfg: &SystemProxyConfig) -> Result<(), Box<dyn Error>> {
    unsafe {
        let flags: u32;
        let address: Option<CString>;
        let bypass: Option<CString>;

        if enabled {
            flags = PROXY_TYPE_DIRECT | PROXY_TYPE_PROXY;
            address = Some(CString::new(&cfg.address as &str)?);
            bypass = Some(CString::new(cfg.excludes.join(";"))?);
        } else {
            flags = PROXY_TYPE_DIRECT;
            address = None;
            bypass = None;
        }

        fn to_pstr_or_null(s: &Option<CString>) -> PSTR {
            PSTR(s.as_ref().map(|a| a.as_ptr()).unwrap_or(null()).cast_mut().cast())
        }

        let mut options = [
            INTERNET_PER_CONN_OPTIONA {
                dwOption: INTERNET_PER_CONN_FLAGS,
                Value: INTERNET_PER_CONN_OPTIONA_0 { dwValue: flags },
            },
            INTERNET_PER_CONN_OPTIONA {
                dwOption: INTERNET_PER_CONN_PROXY_SERVER,
                Value: INTERNET_PER_CONN_OPTIONA_0 {
                    pszValue: to_pstr_or_null(&address),
                },
            },
            INTERNET_PER_CONN_OPTIONA {
                dwOption: INTERNET_PER_CONN_PROXY_BYPASS,
                Value: INTERNET_PER_CONN_OPTIONA_0 {
                    pszValue: to_pstr_or_null(&bypass),
                },
            },
            INTERNET_PER_CONN_OPTIONA {
                dwOption: INTERNET_PER_CONN(INTERNET_PER_CONN_FLAGS_UI),
                Value: INTERNET_PER_CONN_OPTIONA_0 { dwValue: flags },
            },
        ];

        let options = INTERNET_PER_CONN_OPTION_LISTA {
            dwSize: size_of::<INTERNET_PER_CONN_OPTION_LISTA>() as u32,
            pszConnection: PSTR::null(),
            dwOptionCount: options.len() as u32,
            dwOptionError: 0,
            pOptions: (&mut options).as_mut_ptr(),
        };

        if InternetSetOptionA(
            None,
            INTERNET_OPTION_PER_CONNECTION_OPTION,
            Some((&options as *const INTERNET_PER_CONN_OPTION_LISTA).cast::<c_void>()),
            options.dwSize,
        ) == FALSE
        {
            return Err(io::Error::last_os_error().into());
        }
    }

    Ok(())
}

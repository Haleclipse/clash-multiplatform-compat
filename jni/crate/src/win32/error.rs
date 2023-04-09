use std::{
    error,
    fmt::{Debug, Display, Formatter},
};

use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

pub struct Error {
    syscall: String,
    errno: WIN32_ERROR,
}

impl Error {
    pub fn new(syscall: &str, errno: WIN32_ERROR) -> Self {
        Self {
            syscall: syscall.to_owned(),
            errno,
        }
    }

    pub fn with_current(syscall: &str) -> Self {
        Self::new(syscall, unsafe { GetLastError() })
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.syscall, self.errno.to_hresult().message()))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl error::Error for Error {}

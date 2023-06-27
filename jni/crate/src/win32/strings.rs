use std::{iter::once, ops::Deref, string::FromUtf16Error};

pub trait Win32StringIntoExt {
    fn to_win32_utf16(&self) -> Vec<u16>;
}

impl<T: Deref<Target = str>> Win32StringIntoExt for T {
    fn to_win32_utf16(&self) -> Vec<u16> {
        self.encode_utf16().chain(once(0)).collect()
    }
}

pub trait Win32StringFromExt: Sized {
    fn from_win32_utf16(chars: &[u16]) -> Result<Self, FromUtf16Error>;
}

impl<T: From<String>> Win32StringFromExt for T {
    fn from_win32_utf16(chars: &[u16]) -> Result<Self, FromUtf16Error> {
        let null_index = chars.iter().position(|c| *c == 0).unwrap_or(0);

        Ok(Self::from(String::from_utf16(&chars[..null_index])?))
    }
}

pub fn join_arguments(arguments: &[String]) -> String {
    if arguments.is_empty() {
        return "".to_owned();
    }

    let mut ret = String::new();

    for s in arguments {
        ret.push('"');

        for c in s.chars() {
            if c == '"' {
                ret.push('\\');
            }

            ret.push(c);
        }

        ret.push('"');

        ret.push(' ')
    }

    ret
}

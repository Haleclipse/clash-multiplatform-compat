use std::marker::PhantomData;

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HANDLE, HWND},
        UI::WindowsAndMessaging::{GetPropW, SetPropW},
    },
};

use crate::win32::strings::Win32StringIntoExt;

pub struct WindowProp<'a, T: Clone> {
    key: &'a str,
    _data: PhantomData<fn(T) -> T>,
}

impl<'a, T: Clone> WindowProp<'a, T> {
    pub const fn new(key: &'a str) -> WindowProp<'a, T> {
        WindowProp {
            key,
            _data: PhantomData {},
        }
    }

    pub fn get(&self, window: HWND) -> Option<T> {
        unsafe {
            let key = self.key.to_win32_utf16();

            let HANDLE(prop) = GetPropW(window, PCWSTR::from_raw(key.as_ptr()));
            if prop != 0 {
                let value = &*(prop as *mut T).clone();

                Some(value.clone())
            } else {
                None
            }
        }
    }

    pub fn set(&self, window: HWND, value: Option<T>) {
        unsafe {
            let key = self.key.to_win32_utf16();

            let HANDLE(prop) = GetPropW(window, PCWSTR::from_raw(key.as_ptr()));
            if prop != 0 {
                drop(Box::from_raw(prop as *mut T))
            }
            if let Some(value) = value {
                SetPropW(
                    window,
                    PCWSTR::from_raw(key.as_ptr()),
                    HANDLE(Box::into_raw(Box::new(value)) as isize),
                );
            } else {
                SetPropW(window, PCWSTR::from_raw(key.as_ptr()), HANDLE::default());
            }
        }
    }
}

use std::{ffi::CStr, marker::PhantomData};

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HANDLE, HWND},
        UI::WindowsAndMessaging::{GetPropA, SetPropA},
    },
};

pub struct WindowProp<'a, T: Clone> {
    key: &'a CStr,
    _data: PhantomData<fn(T) -> T>,
}

impl<'a, T: Clone> WindowProp<'a, T> {
    pub const fn new(key: &'a CStr) -> WindowProp<'a, T> {
        WindowProp {
            key,
            _data: PhantomData {},
        }
    }

    pub fn get(&self, window: HWND) -> Option<T> {
        unsafe {
            let HANDLE(prop) = GetPropA(window, PCSTR(self.key.as_ptr().cast()));
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
            let key = PCSTR(self.key.as_ptr().cast());

            let HANDLE(prop) = GetPropA(window, key);
            if prop != 0 {
                drop(Box::from_raw(prop as *mut T))
            }
            if let Some(value) = value {
                SetPropA(window, key, HANDLE(Box::into_raw(Box::new(value)) as isize));
            } else {
                SetPropA(window, key, HANDLE::default());
            }
        }
    }
}

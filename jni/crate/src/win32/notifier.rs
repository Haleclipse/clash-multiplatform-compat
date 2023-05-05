use std::{
    ffi::{CStr, CString},
    mem::size_of,
    panic,
    sync::{Arc, Mutex, Once},
};

use cstr::cstr;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{ERROR_BUFFER_OVERFLOW, FALSE, HWND, LPARAM, LRESULT, WIN32_ERROR, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            Shell::*,
            WindowsAndMessaging::{
                AppendMenuA, CreateMenu, CreatePopupMenu, CreateWindowExA, DefWindowProcA, DestroyIcon, DestroyMenu,
                DestroyWindow, GetSystemMetrics, RegisterClassA, SetForegroundWindow, TrackPopupMenuEx, HICON, HMENU, MF_POPUP,
                MF_STRING, SM_MENUDROPALIGNMENT, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTALIGN, TPM_RIGHTBUTTON, WINDOW_EX_STYLE,
                WM_APP, WM_CONTEXTMENU, WM_DESTROY, WNDCLASSA, WS_OVERLAPPEDWINDOW,
            },
        },
    },
};

use crate::{
    common::notifier::{Listener, MenuItem, Notifier},
    win32::{error::Error, icons::load_icon, prop::WindowProp, thread},
};

#[derive(Default)]
pub struct NotifierContext {
    window: HWND,
    icon: HICON,
}

static PROP_MENU: WindowProp<'static, HMENU> = WindowProp::new(cstr!("menu"));

impl NotifierContext {
    unsafe fn insert_menu(menu: HMENU, layout: &[MenuItem]) -> Result<(), Box<dyn std::error::Error>> {
        for item in layout {
            match item {
                MenuItem::Item { title, id } => {
                    let title = CString::new(&title[..])?;
                    if AppendMenuA(menu, MF_STRING, *id as usize, PCSTR(title.as_ptr().cast())) == FALSE {
                        return Err(Error::with_current("AppendMenuA").into());
                    }
                }
                MenuItem::SubMenu { title, items } => {
                    let sub_menu = CreateMenu()?;

                    let title = CString::new(&title[..])?;
                    if AppendMenuA(menu, MF_STRING | MF_POPUP, sub_menu.0 as usize, PCSTR(title.as_ptr().cast())) == FALSE {
                        DestroyMenu(sub_menu);

                        return Err(Error::with_current("AppendMenuA").into());
                    }

                    NotifierContext::insert_menu(sub_menu, items)?;
                }
            }
        }

        Ok(())
    }
}

impl Notifier for NotifierContext {
    fn set_menu(&self, items: Option<&[MenuItem]>) -> Result<(), Box<dyn std::error::Error>> {
        let new_menu = if let Some(items) = items {
            unsafe {
                let new_menu = CreatePopupMenu()?;

                if let Err(err) = NotifierContext::insert_menu(new_menu, items) {
                    DestroyMenu(new_menu);

                    return Err(err);
                }

                Some(new_menu)
            }
        } else {
            None
        };

        unsafe {
            if let Some(menu) = PROP_MENU.get(self.window) {
                DestroyMenu(menu);
            }

            PROP_MENU.set(self.window, new_menu);
        }

        Ok(())
    }
}

impl Drop for NotifierContext {
    fn drop(&mut self) {
        unsafe {
            let mut notify_data = NOTIFYICONDATAA::default();
            notify_data.cbSize = size_of::<NOTIFYICONDATAA>() as u32;
            notify_data.hWnd = self.window;
            notify_data.uFlags = NOTIFY_ICON_DATA_FLAGS::default();
            notify_data.uID = self.window.0 as u32;

            Shell_NotifyIconA(NIM_DELETE, &notify_data);

            DestroyWindow(self.window);
            DestroyIcon(self.icon);
        }
    }
}

static PROP_LISTENER: WindowProp<'static, Arc<Mutex<dyn Listener>>> = WindowProp::new(cstr!("listener"));

unsafe extern "system" fn menu_window_procedure(window: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let listener = if let Some(callback) = PROP_LISTENER.get(window) {
        callback
    } else {
        return DefWindowProcA(window, message, w_param, l_param);
    };

    match message {
        WM_DESTROY => {
            if let Some(menu) = PROP_MENU.get(window) {
                DestroyMenu(menu);
            }

            PROP_LISTENER.set(window, None);
            PROP_MENU.set(window, None);
        }
        WM_APP => {
            match (l_param.0 & 0xffff) as u32 {
                NIN_SELECT => {
                    listener.lock().unwrap().on_active();
                }
                WM_CONTEXTMENU => {
                    if let Some(menu) = PROP_MENU.get(window) {
                        SetForegroundWindow(window);

                        let flags = TPM_RIGHTBUTTON | TPM_RETURNCMD;
                        let flags = if GetSystemMetrics(SM_MENUDROPALIGNMENT) != 0 {
                            flags | TPM_RIGHTALIGN
                        } else {
                            flags | TPM_LEFTALIGN
                        };

                        let x = (w_param.0 & 0xffff) as i16;
                        let y = (w_param.0 >> 16) as i16;

                        let id = TrackPopupMenuEx(menu, flags.0, x as i32, y as i32, window, None);
                        if id.0 != 0 {
                            listener.lock().unwrap().on_menu_active(id.0 as u16);
                        }
                    }
                }
                _ => (),
            };
        }
        _ => (),
    }

    DefWindowProcA(window, message, w_param, l_param)
}

static CLASS_NAME_NOTIFIER: &'static CStr = cstr!("compat-notifier-window");
static ONCE_REGISTER_CLASS: Once = Once::new();

fn register_class() {
    unsafe {
        let mut class = WNDCLASSA::default();
        class.lpfnWndProc = Some(menu_window_procedure);
        class.hInstance = GetModuleHandleA(None).expect("unable to get module name");
        class.lpszClassName = PCSTR(CLASS_NAME_NOTIFIER.as_ptr().cast());

        if RegisterClassA(&class as *const WNDCLASSA) == 0 {
            panic!("{}", Error::with_current("RegisterClassA"));
        }
    }
}

pub fn add_notifier(
    listener: impl Listener + 'static,
    _app_id: &str,
    title: &str,
    icon: &str,
    _is_rtl: bool,
) -> Result<Box<dyn Notifier>, Box<dyn std::error::Error>> {
    ONCE_REGISTER_CLASS.call_once(register_class);

    unsafe {
        let mut notifier = NotifierContext::default();

        notifier.window = thread::run_on_main_thread(|| -> Result<HWND, Error> {
            let current_module = match GetModuleHandleA(None) {
                Ok(module) => module,
                Err(err) => {
                    return Err(Error::new("GetModuleHandleA", WIN32_ERROR(err.code().0 as u32)));
                }
            };

            let window = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                PCSTR(CLASS_NAME_NOTIFIER.as_ptr().cast()),
                None,
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                current_module,
                None,
            );
            if window == HWND::default() {
                return Err(Error::with_current("CreateWindowExA"));
            }

            Ok(window)
        })?;

        PROP_LISTENER.set(notifier.window, Some(Arc::new(Mutex::new(listener))));

        notifier.icon = load_icon(icon)?;

        let tips = CString::new(title)?;
        let tips = tips.as_bytes();
        if tips.len() > 128 {
            return Err(Error::new("NOTIFYICONDATAA", ERROR_BUFFER_OVERFLOW).into());
        }

        let mut notify_data = NOTIFYICONDATAA::default();
        notify_data.cbSize = size_of::<NOTIFYICONDATAA>() as u32;
        notify_data.hWnd = notifier.window;
        notify_data.hIcon = notifier.icon;
        notify_data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        notify_data.uID = notifier.window.0 as u32;
        notify_data.uCallbackMessage = WM_APP;
        notify_data.szTip[..tips.len()].copy_from_slice(tips);
        if Shell_NotifyIconA(NIM_ADD, &notify_data) == FALSE {
            return Err(Error::with_current("Shell_NotifyIconA").into());
        }

        notify_data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        if Shell_NotifyIconA(NIM_SETVERSION, &notify_data) == FALSE {
            return Err(Error::with_current("Shell_NotifyIconA").into());
        }

        Ok(Box::new(notifier))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        common::notifier::Listener,
        win32::{
            icons::install_icon,
            notifier::{add_notifier, MenuItem},
            testdata::TestData,
        },
    };

    struct ListenerImpl {}

    impl Listener for ListenerImpl {
        fn on_active(&self) {
            println!("active");
        }

        fn on_menu_active(&self, id: u16) {
            println!("menu {} clicked", id);
        }
    }

    const TEST_APP_ICON_NAME: &str = "clash-multiplatform-test";
    const TEST_APP_ICON_PATH: &str = "clash-multiplatform.ico";

    #[test]
    pub fn test_show_icon() -> Result<(), Box<dyn std::error::Error>> {
        let icon = TestData::get(TEST_APP_ICON_PATH).unwrap().data;

        install_icon(TEST_APP_ICON_NAME, &icon)?;

        let notifier = add_notifier(ListenerImpl {}, "", "Clash Compat Library", TEST_APP_ICON_NAME, false)?;

        notifier.set_menu(Some(&[
            MenuItem::Item {
                id: 114,
                title: "Item 114".to_owned(),
            },
            MenuItem::SubMenu {
                title: "Sub Items".to_string(),
                items: vec![
                    MenuItem::Item {
                        id: 514,
                        title: "Item 514".to_owned(),
                    },
                    MenuItem::Item {
                        id: 1919,
                        title: "Item 1919".to_owned(),
                    },
                ],
            },
            MenuItem::Item {
                id: 810,
                title: "Item 810".to_owned(),
            },
        ]))?;

        std::thread::sleep(Duration::from_secs(10));

        drop(notifier);

        println!("removed");

        std::thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}

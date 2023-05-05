use std::{
    ffi::c_void,
    mem,
    ptr::null_mut,
    sync::{Arc, Mutex},
};

use cstr::cstr;
use windows::Win32::{
    Foundation::{BOOL, FALSE, HWND, LPARAM, LRESULT, RECT, TRUE, WPARAM},
    Graphics::Dwm::DwmExtendFrameIntoClientArea,
    UI::{
        Controls::MARGINS,
        HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
        WindowsAndMessaging::{
            CallWindowProcA, DefWindowProcA, EnumChildWindows, GetSystemMenu, GetWindowLongPtrA, GetWindowRect, IsZoomed,
            SendMessageA, SetWindowLongA, SetWindowLongPtrA, SetWindowPos, TrackPopupMenu, GWLP_WNDPROC, GWL_STYLE, HTBOTTOM,
            HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTCLIENT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, HTTRANSPARENT,
            NCCALCSIZE_PARAMS, SM_CXPADDEDBORDER, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WM_COMMAND, WM_DESTROY,
            WM_MOVE, WM_NCCALCSIZE, WM_NCHITTEST, WM_NCRBUTTONDOWN, WM_NCRBUTTONUP, WM_SIZE, WM_SYSCOMMAND, WS_THICKFRAME,
        },
    },
};

use crate::{common::window::WindowHints, win32::prop::WindowProp};

const FRAME_EDGE_INSETS: usize = 0;
const FRAME_TITLE_BAR: usize = 1;

type WindowProcedureFunc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

#[derive(Default)]
struct Context {
    root: HWND,

    position: RECT,

    frame_sizes: [u32; 2],
    control_positions: [RECT; 2],
}

impl Context {
    fn match_client_area(&self, global_x: i32, global_y: i32) -> u32 {
        let context_position = self.position;
        let (x, y) = (global_x - context_position.left, global_y - context_position.top);

        for rect in &self.control_positions {
            let rect = rect;
            if rect.left < x && x < rect.right && rect.top < y && y < rect.bottom {
                return HTCLIENT;
            }
        }

        let width = context_position.right - context_position.left;
        let height = context_position.bottom - context_position.top;
        let edge_inset = self.frame_sizes[FRAME_EDGE_INSETS] as i32;

        let in_left = x < edge_inset;
        let in_top = y < edge_inset;
        let in_right = x > width - edge_inset;
        let in_bottom = y > height - edge_inset;

        match () {
            _ if in_top && in_left => HTTOPLEFT,
            _ if in_top && in_right => HTTOPRIGHT,
            _ if in_top => HTTOP,
            _ if in_bottom && in_left => HTBOTTOMLEFT,
            _ if in_bottom && in_right => HTBOTTOMRIGHT,
            _ if in_bottom => HTBOTTOM,
            _ if in_left => HTLEFT,
            _ if in_right => HTRIGHT,
            _ => {
                return if y < self.frame_sizes[FRAME_TITLE_BAR] as i32 {
                    HTCAPTION
                } else {
                    HTCLIENT
                };
            }
        }
    }
}

pub struct Hints {
    context: Arc<Mutex<Context>>,
}

impl WindowHints for Hints {
    fn set_control_position(&self, control_type: usize, left: i32, top: i32, right: i32, bottom: i32) {
        self.context.lock().unwrap().control_positions[control_type] = RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    fn set_frame_size(&self, frame_type: usize, size: u32) {
        self.context.lock().unwrap().frame_sizes[frame_type] = size
    }
}

static PROP_COMPAT_CONTEXT: WindowProp<'static, Arc<Mutex<Context>>> = WindowProp::new(cstr!("compat-context"));
static PROP_AWT_PROCEDURE: WindowProp<'static, WindowProcedureFunc> = WindowProp::new(cstr!("awt-procedure"));

unsafe fn get_caption_padding(handle: HWND) -> i32 {
    if IsZoomed(handle) == TRUE {
        GetSystemMetricsForDpi(SM_CXPADDEDBORDER, GetDpiForWindow(handle))
    } else {
        0
    }
}

unsafe extern "system" fn delegated_window_procedure(window: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let awt_procedure = if let Some(awt_procedure) = PROP_AWT_PROCEDURE.get(window) {
        awt_procedure
    } else {
        return DefWindowProcA(window, message, w_param, l_param);
    };

    let context = if let Some(context) = PROP_COMPAT_CONTEXT.get(window) {
        context
    } else {
        return CallWindowProcA(Some(awt_procedure), window, message, w_param, l_param);
    };

    match message {
        WM_DESTROY => {
            PROP_AWT_PROCEDURE.set(window, None);
            PROP_COMPAT_CONTEXT.set(window, None);
        }
        WM_NCHITTEST => {
            let context = context.lock().unwrap();

            let area = context.match_client_area((l_param.0 & 0xffff) as i16 as i32, (l_param.0 >> 16) as i16 as i32);

            if context.root == window {
                return LRESULT(area as isize);
            }

            if area != HTCLIENT {
                return LRESULT(HTTRANSPARENT as isize);
            }

            return LRESULT(area as isize);
        }
        WM_NCCALCSIZE => {
            let context = context.lock().unwrap();

            if context.root == window && w_param.0 != 0 {
                let l_param = l_param.0 as *mut NCCALCSIZE_PARAMS;
                if l_param != null_mut() {
                    (*l_param).rgrc[0].top += get_caption_padding(window);

                    return LRESULT(0);
                }
            }
        }
        WM_SIZE | WM_MOVE => {
            let mut context = context.lock().unwrap();

            if context.root == window {
                GetWindowRect(window, &mut context.position as *mut RECT);
            }
        }
        WM_NCRBUTTONDOWN => {
            if w_param.0 as u32 == HTCAPTION {
                return LRESULT(0);
            }
        }
        WM_NCRBUTTONUP => {
            let root = context.lock().unwrap().root;

            if w_param.0 as u32 == HTCAPTION {
                let (x, y) = ((l_param.0 & 0xffff) as i16 as i32, (l_param.0 >> 16) as i16 as i32);
                let menu = GetSystemMenu(root, FALSE);

                TrackPopupMenu(menu, Default::default(), x, y, 0, root, None);

                return LRESULT(0);
            }
        }
        WM_COMMAND => {
            if (w_param.0 & 0xf000) != 0 {
                return SendMessageA(window, WM_SYSCOMMAND, w_param, l_param);
            }
        }
        WM_SYSCOMMAND => {
            let root = context.lock().unwrap().root;

            return DefWindowProcA(root, message, w_param, l_param);
        }
        _ => (),
    }

    CallWindowProcA(Some(awt_procedure), window, message, w_param, l_param)
}

pub unsafe extern "system" fn attach_to_window(window: HWND, l_param: LPARAM) -> BOOL {
    if PROP_COMPAT_CONTEXT.get(window).is_some() {
        return TRUE;
    }

    let holder = &*((l_param.0 as *mut c_void) as *const Arc<Mutex<Context>>);
    PROP_COMPAT_CONTEXT.set(window, Some(holder.clone()));

    let awt_procedure = GetWindowLongPtrA(window, GWLP_WNDPROC);
    PROP_AWT_PROCEDURE.set(window, Some(mem::transmute(awt_procedure)));

    let delegated_window_procedure = (delegated_window_procedure as *const c_void) as isize;
    SetWindowLongPtrA(window, GWLP_WNDPROC, delegated_window_procedure);

    EnumChildWindows(window, Some(attach_to_window), l_param);

    SetWindowPos(
        window,
        None,
        0,
        0,
        0,
        0,
        SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
    );

    TRUE
}

pub fn set_borderless(window: i64) -> Result<Box<dyn WindowHints>, Box<dyn std::error::Error>> {
    unsafe {
        let window = HWND(window as isize);

        SetWindowLongA(window, GWL_STYLE, WS_THICKFRAME.0 as i32);

        let margin = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 1,
        };

        DwmExtendFrameIntoClientArea(window, &margin as *const MARGINS)?;

        let mut context = Context {
            root: window,
            position: Default::default(),
            frame_sizes: [0; 2],
            control_positions: [RECT::default(); 2],
        };

        GetWindowRect(context.root, &mut context.position);

        let context = Arc::new(Mutex::new(context));

        attach_to_window(window, LPARAM((&context as *const Arc<Mutex<Context>>) as isize));

        Ok(Box::new(Hints { context }))
    }
}

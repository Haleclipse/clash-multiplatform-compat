use std::{
    collections::HashMap,
    error::Error,
    ffi::c_void,
    mem,
    ptr::{null_mut, slice_from_raw_parts_mut},
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex, RwLock,
    },
};

use cstr::cstr;
use jni_sys::{jclass, JNIEnv, JNINativeMethod, JNI_OK};
use once_cell::sync::Lazy;

use libc::c_long;
use x11::{
    xlib,
    xlib::{
        ClientMessageData, Display, False, SubstructureNotifyMask, SubstructureRedirectMask, True, Window, XClientMessageEvent,
        XCloseDisplay, XDefaultRootWindow, XEvent, XGetWindowAttributes, XInternAtom, XNextEvent, XOpenDisplay, XQueryTree,
        XSendEvent, XWindowAttributes,
    },
};

use crate::{
    common::window::WindowHints,
    helper::{call::jcall, vm::attach_current_thread},
    utils::scoped::Scoped,
};

pub fn is_supported() -> bool {
    return std::env::var("DESKTOP_SESSION")
        .map(|s| s.to_ascii_lowercase() == "gnome")
        .unwrap_or(false);
}

const EDGE_INSETS_INDEX: usize = 0;
const TITLE_BAR_INDEX: usize = 1;

#[derive(Default, Copy, Clone)]
struct Rectangle {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

struct Context {
    root: Window,

    width: u32,
    height: u32,

    control_positions: [Rectangle; 2],
    frame_sizes: [u32; 2],
}

impl Context {
    pub fn set_control_position(&mut self, idx: usize, left: i32, top: i32, right: i32, bottom: i32) {
        self.control_positions[idx] = Rectangle {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn set_frame_size(&mut self, idx: usize, size: u32) {
        self.frame_sizes[idx] = size
    }

    fn new(root: Window, width: u32, height: u32) -> Context {
        Context {
            root,
            width,
            height,
            control_positions: [Rectangle::default(); 2],
            frame_sizes: [0; 2],
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn is_match_title_bar(&self, x: i32, y: i32) -> bool {
        if self
            .control_positions
            .iter()
            .any(|s| s.left <= x && x < s.right && s.top <= y && y < s.bottom)
        {
            return false;
        }

        let insets = self.frame_sizes[EDGE_INSETS_INDEX] as i32;

        insets <= x && x < self.width as i32 - insets && insets <= y && y < self.frame_sizes[TITLE_BAR_INDEX] as i32
    }
}

static TRACKING_WINDOWS: Lazy<RwLock<HashMap<Window, Arc<Mutex<Context>>>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static DELEGATE_INSTALLED: AtomicBool = AtomicBool::new(false);

fn find_tracking_window(window: Window) -> Option<Arc<Mutex<Context>>> {
    TRACKING_WINDOWS.read().unwrap().get(&window).map(|s| s.clone())
}

fn find_tracking_window_if_in_title_bar(window: Window, x: i32, y: i32) -> Option<Arc<Mutex<Context>>> {
    find_tracking_window(window).filter(|ctx| ctx.lock().unwrap().is_match_title_bar(x, y))
}

unsafe extern "C" fn delegated_x_next_event(_: *mut JNIEnv, _: jclass, display: *mut Display, event: *mut XEvent) {
    loop {
        XNextEvent(display, event);

        match (*event).type_ {
            xlib::DestroyNotify => {
                TRACKING_WINDOWS.write().unwrap().remove(&(*event).destroy_window.window);
            }
            xlib::ConfigureNotify => {
                if let Some(context) = find_tracking_window((*event).configure.window) {
                    context
                        .lock()
                        .unwrap()
                        .resize((*event).configure.width as u32, (*event).configure.height as u32);
                }
            }
            xlib::ButtonPress => match (*event).button.button {
                xlib::Button1 => {
                    if let Some(context) =
                        find_tracking_window_if_in_title_bar((*event).button.window, (*event).button.x, (*event).button.y)
                    {
                        let mut request = XClientMessageEvent {
                            type_: xlib::ClientMessage,
                            serial: 0,
                            send_event: 0,
                            display,
                            window: context.lock().unwrap().root,
                            message_type: XInternAtom(display, cstr!("_NET_WM_MOVERESIZE").as_ptr(), True),
                            format: 32,
                            data: ClientMessageData::from([
                                (*event).button.x_root as c_long,
                                (*event).button.y_root as c_long,
                                8, // _NET_WM_MOVERESIZE_MOVE
                                xlib::Button1 as c_long,
                                1, // normal applications
                            ]),
                        };

                        XSendEvent(
                            display,
                            XDefaultRootWindow(display),
                            False,
                            SubstructureNotifyMask | SubstructureRedirectMask,
                            mem::transmute(&mut request),
                        );

                        continue;
                    }
                }
                xlib::Button3 => {
                    if let Some(_) =
                        find_tracking_window_if_in_title_bar((*event).button.window, (*event).button.x, (*event).button.y)
                    {
                        continue;
                    }
                }
                _ => (),
            },
            xlib::ButtonRelease => match (*event).button.button {
                xlib::Button1 => {
                    if let Some(_) =
                        find_tracking_window_if_in_title_bar((*event).button.window, (*event).button.x, (*event).button.y)
                    {
                        continue;
                    }
                }
                xlib::Button3 => {
                    if let Some(context) =
                        find_tracking_window_if_in_title_bar((*event).button.window, (*event).button.x, (*event).button.y)
                    {
                        let mut request = XClientMessageEvent {
                            type_: xlib::ClientMessage,
                            serial: 0,
                            send_event: 0,
                            display,
                            window: context.lock().unwrap().root,
                            message_type: XInternAtom(display, cstr!("_GTK_SHOW_WINDOW_MENU").as_ptr(), True),
                            format: 32,
                            data: ClientMessageData::from([
                                0,
                                (*event).button.x_root as c_long,
                                (*event).button.y_root as c_long,
                                0,
                                0,
                            ]),
                        };

                        XSendEvent(
                            display,
                            XDefaultRootWindow(display),
                            False,
                            SubstructureNotifyMask | SubstructureRedirectMask,
                            mem::transmute(&mut request),
                        );

                        continue;
                    }
                }
                _ => (),
            },
            _ => (),
        }

        break;
    }
}

pub fn install_delegate() {
    let env = attach_current_thread();

    let class = jcall!(*env, FindClass, cstr!("sun/awt/X11/XlibWrapper").as_ptr());

    let methods = [JNINativeMethod {
        name: cstr!("XNextEvent").as_ptr().cast_mut(),
        signature: cstr!("(JJ)V").as_ptr().cast_mut(),
        fnPtr: delegated_x_next_event as *mut c_void,
    }];

    if jcall!(*env, RegisterNatives, class, methods.as_ptr(), 1) != JNI_OK {
        panic!("unsupported jvm")
    }

    DELEGATE_INSTALLED.store(true, Relaxed);
}

fn store_window(
    windows: &mut HashMap<Window, Arc<Mutex<Context>>>,
    context: &Arc<Mutex<Context>>,
    display: *mut Display,
    window: Window,
) {
    if windows.contains_key(&window) {
        return;
    }

    windows.insert(window, context.clone());

    let mut root: Window = Window::default();
    let mut parent: Window = Window::default();
    let mut children: *mut Window = null_mut();
    let mut children_length: u32 = 0;

    unsafe {
        XQueryTree(display, window, &mut root, &mut parent, &mut children, &mut children_length);
    }

    if children != null_mut() {
        unsafe {
            for window in &*slice_from_raw_parts_mut(children, children_length as usize) {
                store_window(windows, context, display, *window)
            }
        }
    }
}

pub fn set_borderless(window: i64) -> Result<Box<dyn WindowHints>, Box<dyn Error>> {
    if !DELEGATE_INSTALLED.load(Relaxed) {
        return Err("not installed".into());
    }
    unsafe {
        let display = XOpenDisplay(null_mut());
        if display == null_mut() {
            return Err("unable to open display".into());
        }
        let display = Scoped::new(display, |d| {
            XCloseDisplay(*d);
        });

        let mut attributes: XWindowAttributes = mem::zeroed();
        XGetWindowAttributes(*display, window as Window, &mut attributes);

        let mut windows = TRACKING_WINDOWS.write()?;
        let context = Arc::new(Mutex::new(Context::new(
            window as Window,
            attributes.width as u32,
            attributes.height as u32,
        )));

        store_window(&mut windows, &context, *display, window as Window);

        struct Hints {
            context: Arc<Mutex<Context>>,
        }

        impl WindowHints for Hints {
            fn set_control_position(&self, idx: usize, left: i32, top: i32, right: i32, bottom: i32) {
                self.context
                    .lock()
                    .unwrap()
                    .set_control_position(idx, left, top, right, bottom);
            }

            fn set_frame_size(&self, idx: usize, size: u32) {
                self.context.lock().unwrap().set_frame_size(idx, size);
            }
        }

        Ok(Box::new(Hints { context }))
    }
}

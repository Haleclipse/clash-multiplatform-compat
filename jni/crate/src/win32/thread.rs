use std::{
    ffi::c_void,
    sync::{Mutex, MutexGuard, Once},
};

use windows::Win32::{
    Foundation::{FALSE, LPARAM, WPARAM},
    System::Threading::{CreateThread, THREAD_CREATION_FLAGS},
    UI::{
        HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE},
        WindowsAndMessaging::{
            DispatchMessageA, GetMessageA, PeekMessageA, PostThreadMessageA, TranslateMessage, MSG, PM_NOREMOVE, WM_USER,
        },
    },
};

const WM_USER_RUN_FUNC: u32 = WM_USER + 8;

unsafe extern "system" fn main_thread_routine(arg: *mut c_void) -> u32 {
    let lock = Box::from_raw(arg as *mut MutexGuard<()>);

    SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE);

    let mut msg = MSG::default();
    PeekMessageA(&mut msg, None, 0, 0, PM_NOREMOVE);
    drop(lock);

    while GetMessageA(&mut msg, None, 0, 0) != FALSE {
        match msg.message {
            WM_USER_RUN_FUNC => {
                Box::from_raw(msg.lParam.0 as *mut Box<dyn FnOnce()>)();

                continue;
            }
            _ => (),
        }

        TranslateMessage(&msg);
        DispatchMessageA(&msg);
    }

    0
}

static mut MAIN_THREAD_THREAD_ID: u32 = 0;

fn init_main_thread() {
    let initialize_lock = Mutex::new(());

    let remote_callback = Box::new(initialize_lock.lock().unwrap());

    unsafe {
        CreateThread(
            None,
            1024,
            Some(main_thread_routine),
            Some(Box::into_raw(remote_callback) as *mut c_void),
            THREAD_CREATION_FLAGS::default(),
            Some(&mut MAIN_THREAD_THREAD_ID),
        )
        .expect("unable to create main thread");
    }

    drop(initialize_lock.lock().expect("unable to lock")); // Await thread initialize
}

static ONCE_INIT_MAIN_THREAD: Once = Once::new();

pub fn run_on_main_thread<R: Send, F: (FnOnce() -> R) + Send>(f: F) -> R {
    ONCE_INIT_MAIN_THREAD.call_once(init_main_thread);

    let result: Mutex<Option<R>> = Mutex::new(None);

    let mut runnable_lock = result.lock().unwrap();
    let runnable: Box<dyn FnOnce()> = Box::new(move || *runnable_lock = Some(f()));

    unsafe {
        PostThreadMessageA(
            MAIN_THREAD_THREAD_ID,
            WM_USER_RUN_FUNC,
            WPARAM::default(),
            LPARAM(Box::into_raw(Box::new(runnable)) as isize),
        )
        .expect("unable to post message to main thread");
    }

    let mut result = result.lock().unwrap();

    result.take().unwrap()
}

#[cfg(test)]
mod tests {
    use windows::Win32::System::Threading::GetCurrentThreadId;

    use crate::win32::thread::{run_on_main_thread, MAIN_THREAD_THREAD_ID};

    #[test]
    pub fn test_run_on_main_thread() {
        let tid = run_on_main_thread(|| unsafe { GetCurrentThreadId() });

        assert_eq!(tid, unsafe { MAIN_THREAD_THREAD_ID });
    }
}

use std::ffi::c_void;

use jni_sys::{jint, JavaVM, JNI_VERSION_1_8};

use crate::helper::vm::install_java_vm;

mod file;
mod notification;
mod notifier;
mod process;
mod shell;
mod theme;
mod window;

mod helper;
mod utils;

mod common;

#[cfg(windows)]
mod win32;

#[cfg(target_os = "linux")]
mod linux;

#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: *mut JavaVM, _: *mut c_void) -> jint {
    install_java_vm(vm);

    #[cfg(target_os = "linux")]
    linux::window::install_delegate();

    #[cfg(debug_assertions)]
    {
        use std::io::Write;

        let pid = std::process::id();

        std::io::stderr().write_fmt(format_args!("Process = {pid}\n")).ok();
    }

    JNI_VERSION_1_8
}

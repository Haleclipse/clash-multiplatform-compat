use jni_sys::{jboolean, jclass, jint, jlong, JNIEnv, JNI_TRUE};

use crate::{common::window::WindowHints, helper::throwable::rethrow_java_io_exception};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_WindowCompat_nativeIsSupported(_: *mut JNIEnv, _: jclass) -> jboolean {
    #[cfg(windows)]
    return JNI_TRUE;

    #[cfg(target_os = "linux")]
    return if crate::linux::window::is_supported() {
        JNI_TRUE
    } else {
        jni_sys::JNI_FALSE
    };
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_WindowCompat_nativeSetBorderless(
    env: *mut JNIEnv,
    _: jclass,
    window: jlong,
) -> jlong {
    rethrow_java_io_exception(env, || {
        #[cfg(windows)]
        let context = crate::win32::window::set_borderless(window)?;

        #[cfg(target_os = "linux")]
        let context = crate::linux::window::set_borderless(window)?;

        Ok(Box::into_raw(Box::new(context)) as jlong)
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_WindowCompat_nativeContextSetFrameSize(
    _: *mut JNIEnv,
    _: jclass,
    ptr: jlong,
    frame_type: jint,
    size: jint,
) {
    let hints = unsafe { &*(ptr as *mut Box<dyn WindowHints>) };

    hints.set_frame_size(frame_type as usize, size as u32);
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_WindowCompat_nativeContextSetControlPosition(
    _: *mut JNIEnv,
    _: jclass,
    ptr: jlong,
    control_type: jint,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    let hints = unsafe { &*(ptr as *mut Box<dyn WindowHints>) };

    hints.set_control_position(control_type as usize, left, top, right, bottom);
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_WindowCompat_nativeContextRelease(_: *mut JNIEnv, _: jclass, ptr: jlong) {
    unsafe {
        drop(Box::from_raw(ptr as *mut Box<dyn WindowHints>));
    }
}

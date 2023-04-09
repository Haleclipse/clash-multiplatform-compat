use jni_sys::{jboolean, jclass, jstring, JNIEnv, JNI_TRUE};

use crate::helper::{strings::java_string_to_string, throwable::rethrow_java_io_exception};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotificationCompat_nativeIsSupported(_: *mut JNIEnv, _: jclass) -> jboolean {
    #[cfg(windows)]
    return JNI_TRUE;

    #[cfg(target_os = "linux")]
    return if crate::linux::notification::is_supported() {
        JNI_TRUE
    } else {
        jni_sys::JNI_FALSE
    };
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotificationCompat_nativeSendNotification(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
    title: jstring,
    message: jstring,
) {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);
        let title = java_string_to_string(env, title);
        let message = java_string_to_string(env, message);

        #[cfg(windows)]
        return crate::win32::notification::send_notification(&app_id, &title, &message);

        #[cfg(target_os = "linux")]
        return crate::linux::notification::send_notification(&app_id, &title, &message);
    });
}

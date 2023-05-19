use crate::{
    common::network::SystemProxyConfig,
    helper::{array::iterate_object_array, strings::java_string_to_string, throwable::rethrow_java_io_exception},
};
use jni_sys::{jboolean, jclass, jobjectArray, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NetworkCompat_nativeIsSystemProxySupported(
    _: *mut JNIEnv,
    _: jclass,
) -> jboolean {
    #[cfg(target_os = "linux")]
    return {
        if crate::linux::network::is_system_proxy_supported() {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    };

    #[cfg(windows)]
    return JNI_TRUE;
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NetworkCompat_nativeSetSystemProxy(
    env: *mut JNIEnv,
    _: jclass,
    enabled: jboolean,
    address: jstring,
    excludes: jobjectArray,
) {
    rethrow_java_io_exception(env, || {
        let enabled = enabled != JNI_FALSE;
        let cfg = SystemProxyConfig {
            address: java_string_to_string(env, address),
            excludes: iterate_object_array(env, excludes)
                .map(|s| java_string_to_string(env, s))
                .collect::<Vec<_>>(),
        };

        #[cfg(target_os = "linux")]
        crate::linux::network::set_system_proxy(enabled, &cfg)?;

        #[cfg(windows)]
        crate::win32::network::set_system_proxy(enabled, &cfg)?;

        Ok(())
    });
}

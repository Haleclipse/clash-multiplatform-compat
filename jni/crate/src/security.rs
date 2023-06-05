use jni_sys::{jclass, jint, jstring, JNIEnv};
use std::ptr::null_mut;

#[no_mangle]
#[cfg(target_os = "linux")]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetUnixUid(_: *mut JNIEnv, _: jclass) -> jint {
    crate::linux::security::get_uid() as jint
}

#[no_mangle]
#[cfg(target_os = "linux")]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetUnixGid(_: *mut JNIEnv, _: jclass) -> jint {
    crate::linux::security::get_gid() as jint
}

#[no_mangle]
#[cfg(target_os = "linux")]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetSELinuxContext(
    env: *mut JNIEnv,
    _: jclass,
) -> jstring {
    crate::helper::throwable::rethrow_java_io_exception(env, || {
        Ok(crate::helper::strings::string_to_java_string(
            env,
            &crate::linux::security::get_selinux_context()?,
        ))
    })
    .unwrap_or(null_mut())
}

#[cfg(windows)]
fn throw_unsupported(env: *mut JNIEnv) {
    let clazz = crate::helper::call::jcall!(
        env,
        FindClass,
        cstr::cstr!("java/lang/UnsupportedOperationException").as_ptr()
    );

    crate::helper::call::jcall!(env, ThrowNew, clazz, cstr::cstr!("Unsupported platform").as_ptr());
}

#[no_mangle]
#[cfg(windows)]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetUnixUid(env: *mut JNIEnv, _: jclass) -> jint {
    throw_unsupported(env);

    -1
}

#[no_mangle]
#[cfg(windows)]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetUnixGid(env: *mut JNIEnv, _: jclass) -> jint {
    throw_unsupported(env);

    -1
}

#[no_mangle]
#[cfg(windows)]
pub extern "C" fn Java_com_github_kr328_clash_compat_SecurityCompat_nativeGetSELinuxContext(
    env: *mut JNIEnv,
    _: jclass,
) -> jstring {
    throw_unsupported(env);

    null_mut()
}

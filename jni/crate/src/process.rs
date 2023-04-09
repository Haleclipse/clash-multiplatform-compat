use std::ptr::null_mut;

use jni_sys::{jclass, jint, jlong, jobject, jobjectArray, jstring, JNIEnv};

use crate::{
    common::file::FileDescriptor,
    file::get_file_descriptor,
    helper::{array::iterate_object_array, strings::java_string_to_string, throwable::rethrow_java_io_exception},
};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ProcessCompat_nativeCreateProcess(
    env: *mut JNIEnv,
    _: jclass,
    executable: jstring,
    arguments: jobjectArray,
    working_dir: jstring,
    environments: jobjectArray,
    extra_fds: jobjectArray,
    fd_stdin: jobject,
    fd_stdout: jobject,
    fd_stderr: jobject,
) -> jlong {
    rethrow_java_io_exception(env, || {
        let executable = java_string_to_string(env, executable);
        let arguments = iterate_object_array(env, arguments)
            .map(|o| java_string_to_string(env, o))
            .collect::<Vec<_>>();
        let working_dir = java_string_to_string(env, working_dir);
        let environments = iterate_object_array(env, environments)
            .map(|o| java_string_to_string(env, o))
            .collect::<Vec<_>>();
        let extra_fds = iterate_object_array(env, extra_fds)
            .map(|fd| get_file_descriptor(env, fd))
            .collect::<Vec<_>>();
        let fd_stdin = if fd_stdin != null_mut() {
            Some(get_file_descriptor(env, fd_stdin))
        } else {
            None
        };
        let fd_stdout = if fd_stdout != null_mut() {
            Some(get_file_descriptor(env, fd_stdout))
        } else {
            None
        };
        let fd_stderr = if fd_stderr != null_mut() {
            Some(get_file_descriptor(env, fd_stderr))
        } else {
            None
        };

        #[cfg(windows)]
        return crate::win32::process::create_process(
            &executable,
            &arguments,
            &working_dir,
            &environments,
            &extra_fds,
            fd_stdin,
            fd_stdout,
            fd_stderr,
        );

        #[cfg(target_os = "linux")]
        return crate::linux::process::create_process(
            &executable,
            &arguments[..],
            &working_dir,
            &environments[..],
            &extra_fds,
            fd_stdin,
            fd_stdout,
            fd_stderr,
        );
    })
    .unwrap_or(-1) as jlong
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ProcessCompat_nativeWaitProcess(
    _: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) -> jint {
    #[cfg(windows)]
    return crate::win32::process::wait_process(handle as FileDescriptor);

    #[cfg(target_os = "linux")]
    return crate::linux::process::wait_process(handle as FileDescriptor) as jint;
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ProcessCompat_nativeKillProcess(_: *mut JNIEnv, _: jclass, handle: jlong) {
    #[cfg(windows)]
    crate::win32::process::kill_process(handle as FileDescriptor);

    #[cfg(target_os = "linux")]
    crate::linux::process::kill_process(handle as FileDescriptor);
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ProcessCompat_nativeReleaseProcess(
    _: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) {
    #[cfg(windows)]
    crate::win32::process::release_process(handle as FileDescriptor);

    #[cfg(target_os = "linux")]
    let _ = handle;
}

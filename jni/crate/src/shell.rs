use std::ptr::null_mut;

use jni_sys::{jboolean, jbyteArray, jclass, jlong, jmethodID, jobjectArray, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::{
    common::shell::FileFilter,
    helper::{
        array::{collect_java_bytes, iterate_object_array},
        call::jcall,
        lazy::{JRef, LazyJRef},
        strings::{java_string_to_string, string_to_java_string},
        throwable::rethrow_java_io_exception,
    },
};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeIsSupported(_: *mut JNIEnv, _: jclass) -> jboolean {
    #[cfg(windows)]
    return JNI_TRUE;

    #[cfg(target_os = "linux")]
    return if crate::linux::shell::is_supported() {
        JNI_TRUE
    } else {
        JNI_FALSE
    };
}

static M_PICKER_FILTER_NAME: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/ShellCompat$NativePickerFilter",
        "name",
        "()Ljava/lang/String;",
    ))
});
static M_PICKER_FILTER_EXTENSIONS: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/ShellCompat$NativePickerFilter",
        "extensions",
        "()[Ljava/lang/String;",
    ))
});

fn java_filters_to_filters(env: *mut JNIEnv, filters: jobjectArray) -> Vec<FileFilter> {
    iterate_object_array(env, filters)
        .map(|filter| {
            let name = jcall!(env, CallObjectMethod, filter, *M_PICKER_FILTER_NAME.get()) as jstring;
            let extensions = jcall!(env, CallObjectMethod, filter, *M_PICKER_FILTER_EXTENSIONS.get()) as jobjectArray;

            let name = java_string_to_string(env, name);
            let extensions = iterate_object_array(env, extensions)
                .map(|ext| java_string_to_string(env, ext))
                .collect::<Vec<String>>();

            FileFilter { label: name, extensions }
        })
        .collect::<_>()
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeRunPickFile(
    env: *mut JNIEnv,
    _: jclass,
    window: jlong,
    title: jstring,
    filters: jobjectArray,
) -> jstring {
    rethrow_java_io_exception(env, || {
        let filters: Vec<FileFilter> = java_filters_to_filters(env, filters);

        #[cfg(windows)]
        let result = crate::win32::shell::run_pick_file(window, &java_string_to_string(env, title), &filters)?;

        #[cfg(target_os = "linux")]
        let result = crate::linux::shell::run_pick_file(window, &java_string_to_string(env, title), &filters)?;

        Ok(match result {
            None => null_mut(),
            Some(path) => string_to_java_string(env, path.to_str().unwrap()),
        })
    })
    .unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeRunSaveFile(
    env: *mut JNIEnv,
    _: jclass,
    window_handle: jlong,
    file_name: jstring,
    title: jstring,
    filters: jobjectArray,
) -> jstring {
    rethrow_java_io_exception(env, || {
        let filters: Vec<FileFilter> = java_filters_to_filters(env, filters);

        #[cfg(windows)]
        let result = crate::win32::shell::run_save_file(
            window_handle,
            &java_string_to_string(env, file_name),
            &java_string_to_string(env, title),
            &filters,
        )?;

        #[cfg(target_os = "linux")]
        let result = crate::linux::shell::run_save_file(
            window_handle,
            &java_string_to_string(env, file_name),
            &java_string_to_string(env, title),
            &filters,
        )?;

        Ok(match result {
            None => null_mut(),
            Some(path) => string_to_java_string(env, path.to_str().unwrap()),
        })
    })
    .unwrap_or(null_mut())
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeRunLaunchFile(
    env: *mut JNIEnv,
    _: jclass,
    window: jlong,
    file: jstring,
) {
    rethrow_java_io_exception(env, || {
        let file = java_string_to_string(env, file);

        #[cfg(windows)]
        crate::win32::shell::run_launch_file(window, &file)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::run_launch_file(window, &file)?;

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeInstallIcon(
    env: *mut JNIEnv,
    _: jclass,
    name: jstring,
    data: jbyteArray,
) {
    rethrow_java_io_exception(env, || {
        let name = java_string_to_string(env, name);
        let data = collect_java_bytes(env, data);

        #[cfg(windows)]
        crate::win32::shell::install_icon(&name, &data)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::install_icon(&name, &data)?;

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeInstallShortcut(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
    app_name: jstring,
    icon_name: jstring,
    exe_path: jstring,
    args: jobjectArray,
) {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);
        let app_name = java_string_to_string(env, app_name);
        let icon_name = java_string_to_string(env, icon_name);
        let exe_path = java_string_to_string(env, exe_path);
        let args = iterate_object_array(env, args)
            .map(|o| java_string_to_string(env, o))
            .collect::<Vec<String>>();

        #[cfg(windows)]
        crate::win32::shell::install_shortcut(&app_id, &app_name, &icon_name, &exe_path, &args)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::install_shortcut(&app_id, &app_name, &icon_name, &exe_path, &args)?;

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeUninstallShortcut(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
    app_name: jstring,
) {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);
        let app_name = java_string_to_string(env, app_name);

        #[cfg(windows)]
        crate::win32::shell::uninstall_shortcut(&app_id, &app_name)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::uninstall_shortcut(&app_id, &app_name)?;

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeIsRunOnBootExisted(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
) -> jboolean {
    let app_id = java_string_to_string(env, app_id);

    #[cfg(windows)]
    let ret = crate::win32::shell::is_run_on_boot_existed(&app_id);

    #[cfg(target_os = "linux")]
    let ret = crate::linux::shell::is_run_on_boot_existed(&app_id);

    if ret {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeSetRunOnBoot(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
    executable_path: jstring,
    arguments: jobjectArray,
) {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);
        let executable_path = java_string_to_string(env, executable_path);
        let arguments = iterate_object_array(env, arguments)
            .map(|obj| java_string_to_string(env, obj))
            .collect::<Vec<_>>();

        #[cfg(windows)]
        crate::win32::shell::set_run_on_boot(&app_id, &executable_path, &arguments)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::set_run_on_boot(&app_id, &executable_path, &arguments)?;

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ShellCompat_nativeRemoveRunOnBoot(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
) {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);

        #[cfg(windows)]
        crate::win32::shell::remove_run_on_boot(&app_id)?;

        #[cfg(target_os = "linux")]
        crate::linux::shell::remove_run_on_boot(&app_id)?;

        Ok(())
    });
}

use jni_sys::{jboolean, jclass, jlong, jmethodID, jobject, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::{
    common::theme::{Holder, Listener},
    helper::{
        call::jcall,
        lazy::{JRef, LazyJRef},
        refs::GlobalRef,
        throwable::rethrow_java_io_exception,
        vm::attach_current_thread,
    },
};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ThemeCompat_nativeIsSupported(_: *mut JNIEnv, _: jclass) -> jboolean {
    #[cfg(windows)]
    return JNI_TRUE;

    #[cfg(target_os = "linux")]
    return if crate::linux::theme::is_supported() {
        JNI_TRUE
    } else {
        JNI_FALSE
    };
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ThemeCompat_nativeIsNight(env: *mut JNIEnv, _: jclass) -> jboolean {
    rethrow_java_io_exception(env, || {
        #[cfg(windows)]
        let is_night = crate::win32::theme::is_night_mode()?;

        #[cfg(target_os = "linux")]
        let is_night = crate::linux::theme::is_night_mode()?;

        if is_night {
            Ok(JNI_TRUE)
        } else {
            Ok(JNI_FALSE)
        }
    })
    .unwrap_or(JNI_FALSE)
}

static M_ON_THEME_CHANGED_LISTENER_ON_CHANGED: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/ThemeCompat$OnThemeChangedListener",
        "onChanged",
        "(Z)V",
    ))
});

struct ListenerImpl {
    listener_ref: GlobalRef,
}

impl Listener for ListenerImpl {
    fn on_changed(&self, is_night: bool) {
        let env = attach_current_thread();
        let v = if is_night { JNI_TRUE } else { JNI_FALSE };

        jcall!(
            *env,
            CallVoidMethod,
            *self.listener_ref,
            *M_ON_THEME_CHANGED_LISTENER_ON_CHANGED.get(),
            v as u32
        );
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ThemeCompat_nativeAddListener(
    env: *mut JNIEnv,
    _: jclass,
    listener: jobject,
) -> jlong {
    let listener_ref = GlobalRef::new(listener);

    rethrow_java_io_exception(env, move || {
        let listener = ListenerImpl { listener_ref };

        #[cfg(windows)]
        let token = crate::win32::theme::add_night_mode_listener(listener)?;

        #[cfg(target_os = "linux")]
        let token = crate::linux::theme::add_night_mode_listener(listener)?;

        Ok(Box::into_raw(Box::new(token)) as jlong)
    })
    .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_ThemeCompat_nativeReleaseListener(_: *mut JNIEnv, _: jclass, token: jlong) {
    unsafe { drop(Box::from_raw(token as *mut Box<dyn Holder>)) }
}

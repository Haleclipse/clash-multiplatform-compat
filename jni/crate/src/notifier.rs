use std::ptr::null_mut;

use jni_sys::{jboolean, jclass, jint, jlong, jmethodID, jobject, jobjectArray, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::{
    common::notifier::{Listener, MenuItem, Notifier},
    helper::{
        array::iterate_object_array,
        call::jcall,
        lazy::{JRef, LazyJRef},
        refs::GlobalRef,
        strings::java_string_to_string,
        throwable::rethrow_java_io_exception,
        vm::attach_current_thread,
    },
};

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotifierCompat_nativeIsSupported(_: *mut JNIEnv, _: jclass) -> jboolean {
    #[cfg(windows)]
    return JNI_TRUE;

    #[cfg(target_os = "linux")]
    return if crate::linux::notifier::is_supported() {
        JNI_TRUE
    } else {
        JNI_FALSE
    };
}

static M_LISTENER_ON_ACTIVE: LazyJRef<jmethodID> =
    LazyJRef::new(|| JRef::from(("com/github/kr328/clash/compat/NotifierCompat$Listener", "onActive", "()V")));

static M_LISTENER_ON_MENU_ACTIVE: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/NotifierCompat$Listener",
        "onMenuActive",
        "(S)V",
    ))
});

struct ListenerImpl {
    listener_ref: GlobalRef,
}

impl Listener for ListenerImpl {
    fn on_active(&self) {
        let env = attach_current_thread();

        jcall!(*env, CallVoidMethod, *self.listener_ref, *M_LISTENER_ON_ACTIVE.get());
    }

    fn on_menu_active(&self, id: u16) {
        let env = attach_current_thread();

        jcall!(
            *env,
            CallVoidMethod,
            *self.listener_ref,
            *M_LISTENER_ON_MENU_ACTIVE.get(),
            id as jint
        );
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotifierCompat_nativeAdd(
    env: *mut JNIEnv,
    _: jclass,
    app_id: jstring,
    title: jstring,
    icon_name: jstring,
    is_rtl: jboolean,
    listener: jobject,
) -> jlong {
    rethrow_java_io_exception(env, || {
        let app_id = java_string_to_string(env, app_id);
        let title = java_string_to_string(env, title);
        let icon_name = java_string_to_string(env, icon_name);
        let listener = ListenerImpl {
            listener_ref: GlobalRef::new(listener),
        };

        #[cfg(windows)]
        let notifier = crate::win32::notifier::add_notifier(listener, &app_id, &title, &icon_name, is_rtl != JNI_FALSE)?;

        #[cfg(target_os = "linux")]
        let notifier = crate::linux::notifier::add_notifier(listener, &app_id, &title, &icon_name, is_rtl != JNI_FALSE)?;

        Ok(Box::into_raw(Box::new(notifier)) as jlong)
    })
    .unwrap_or(-1)
}

static M_MENU_ITEM_TITLE: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/NotifierCompat$NativeMenuItem",
        "title",
        "()Ljava/lang/String;",
    ))
});

static M_MENU_ITEM_ID: LazyJRef<jmethodID> =
    LazyJRef::new(|| JRef::from(("com/github/kr328/clash/compat/NotifierCompat$NativeMenuItem", "id", "()S")));

static M_MENU_ITEM_SUB_ITEMS: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        "com/github/kr328/clash/compat/NotifierCompat$NativeMenuItem",
        "subItems",
        "()[Lcom/github/kr328/clash/compat/NotifierCompat$NativeMenuItem;",
    ))
});

fn native_menu_item_to_items(env: *mut JNIEnv, items: jobjectArray) -> Vec<MenuItem> {
    iterate_object_array(env, items)
        .map(|obj| {
            let title = jcall!(env, CallObjectMethod, obj, *M_MENU_ITEM_TITLE.get());
            let title = java_string_to_string(env, title as jstring);

            let id = jcall!(env, CallShortMethod, obj, *M_MENU_ITEM_ID.get());
            if id >= 0 {
                MenuItem::Item { title, id: id as u16 }
            } else {
                let items = jcall!(env, CallObjectMethod, obj, *M_MENU_ITEM_SUB_ITEMS.get());

                MenuItem::SubMenu {
                    title,
                    items: native_menu_item_to_items(env, items),
                }
            }
        })
        .collect()
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotifierCompat_nativeSetMenu(
    env: *mut JNIEnv,
    _: jclass,
    ptr: jlong,
    items: jobjectArray,
) {
    rethrow_java_io_exception(env, || {
        unsafe {
            let notifier = &*(ptr as *mut Box<dyn Notifier>);

            if items != null_mut() {
                let items = native_menu_item_to_items(env, items);

                notifier.set_menu(Some(&items))?
            } else {
                notifier.set_menu(None)?;
            }
        }

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_NotifierCompat_nativeRemove(_: *mut JNIEnv, _: jclass, ptr: jlong) {
    unsafe { drop(Box::from_raw(ptr as *mut Box<dyn Notifier>)) }
}

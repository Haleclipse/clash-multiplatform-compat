use crate::helper::{call::jcall, lazy::JRef};
use jni_sys::{jclass, jfieldID, jmethodID, jobject, jstring, JNIEnv, JNI_TRUE};
use std::ptr::null_mut;

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_AppCompat_nativeSetProcessApplicationID(
    env: *mut JNIEnv,
    _: jclass,
    application_id: jstring,
) {
    let c_toolkit: JRef<jclass> = ("java/awt/Toolkit").into();
    let m_toolkit_default: JRef<jmethodID> = (*c_toolkit, "getDefaultToolkit", "()Ljava/awt/Toolkit;", ()).into();
    let o_toolkit_default: jobject = jcall!(env, CallStaticObjectMethod, *c_toolkit, *m_toolkit_default);

    let c_x_toolkit: JRef<jclass> = ("sun/awt/X11/XToolkit").into();
    if *c_x_toolkit != null_mut() {
        if jcall!(env, IsInstanceOf, o_toolkit_default, *c_x_toolkit) == JNI_TRUE {
            let f_app_name: JRef<jfieldID> = (*c_x_toolkit, "awtAppClassName", "Ljava/lang/String;", ()).into();
            jcall!(env, SetStaticObjectField, *c_x_toolkit, *f_app_name, application_id);
        }
    }

    jcall!(env, ExceptionClear);
}

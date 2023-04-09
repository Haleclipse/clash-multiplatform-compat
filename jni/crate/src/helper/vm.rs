use std::{
    ffi::c_void,
    ops::Deref,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering::Relaxed},
};

use jni_sys::{JNIEnv, JavaVM, JNI_OK, JNI_VERSION_1_8};

use crate::helper::call::jcall;

static GLOBAL_JAVA_VM: AtomicPtr<JavaVM> = AtomicPtr::new(null_mut());

pub fn install_java_vm(vm: *mut JavaVM) {
    GLOBAL_JAVA_VM
        .compare_exchange(null_mut(), vm, Relaxed, Relaxed)
        .expect("double install java vm");
}

pub fn current_java_vm() -> *mut JavaVM {
    let ret = GLOBAL_JAVA_VM.load(Relaxed);

    if ret == null_mut() {
        panic!("java runtime not found")
    }

    ret
}

enum CleanType {
    DetachThread,
    PopupFrame,
}

pub struct LocalEnvGuard {
    pub env: *mut JNIEnv,
    clean_type: CleanType,
}

impl Drop for LocalEnvGuard {
    fn drop(&mut self) {
        match self.clean_type {
            CleanType::DetachThread => {
                if jcall!(current_java_vm(), DetachCurrentThread) != JNI_OK {
                    panic!("unable to detach thread")
                }
            }
            CleanType::PopupFrame => {
                jcall!(self.env, PopLocalFrame, null_mut());
            }
        }
    }
}

impl Deref for LocalEnvGuard {
    type Target = *mut JNIEnv;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

pub fn attach_current_thread() -> LocalEnvGuard {
    let vm: *mut JavaVM = current_java_vm();
    let mut env: *mut JNIEnv = null_mut();

    if jcall!(
        vm,
        GetEnv,
        ((&mut env) as *mut *mut JNIEnv) as *mut *mut c_void,
        JNI_VERSION_1_8
    ) == JNI_OK
    {
        if jcall!(env, PushLocalFrame, 32) != JNI_OK {
            panic!("out of memory")
        }

        LocalEnvGuard {
            env,
            clean_type: CleanType::PopupFrame,
        }
    } else {
        if jcall!(
            vm,
            AttachCurrentThread,
            ((&mut env) as *mut *mut JNIEnv) as *mut *mut c_void,
            null_mut()
        ) != JNI_OK
        {
            panic!("unable to attach thread")
        }

        LocalEnvGuard {
            env,
            clean_type: CleanType::DetachThread,
        }
    }
}

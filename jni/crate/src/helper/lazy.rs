use std::{ffi::CString, mem, ops::Deref, sync::Mutex};

use jni_sys::{jclass, jfieldID, jmethodID, jobject};

use crate::helper::{call::jcall, vm::attach_current_thread};

pub trait JValue: Copy {
    fn to_global_ref(self) -> Self;
}

impl JValue for jobject {
    fn to_global_ref(self) -> Self {
        let env = attach_current_thread();

        jcall!(*env, NewGlobalRef, self) as jobject
    }
}

impl JValue for jmethodID {
    fn to_global_ref(self) -> Self {
        self
    }
}

impl JValue for jfieldID {
    fn to_global_ref(self) -> Self {
        self
    }
}

pub struct JRef<T: JValue>(pub T);

impl<T: JValue> Deref for JRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: JValue> From<T> for JRef<T> {
    fn from(value: T) -> Self {
        JRef(value)
    }
}

impl From<&str> for JRef<jclass> {
    fn from(value: &str) -> Self {
        let env = attach_current_thread();

        let name = CString::new(value).unwrap();
        JRef(jcall!(*env, FindClass, name.as_ptr()))
    }
}

impl<C: Into<JRef<jclass>>> From<(C, &str, &str)> for JRef<jmethodID> {
    fn from(value: (C, &str, &str)) -> Self {
        let env = attach_current_thread();

        let name = CString::new(value.1).unwrap();
        let signature = CString::new(value.2).unwrap();
        JRef(jcall!(*env, GetMethodID, value.0.into().0, name.as_ptr(), signature.as_ptr()))
    }
}

impl<C: Into<JRef<jclass>>> From<(C, &str, &str, ())> for JRef<jmethodID> {
    fn from(value: (C, &str, &str, ())) -> Self {
        let env = attach_current_thread();

        let name = CString::new(value.1).unwrap();
        let signature = CString::new(value.2).unwrap();
        JRef(jcall!(
            *env,
            GetStaticMethodID,
            value.0.into().0,
            name.as_ptr(),
            signature.as_ptr()
        ))
    }
}

impl<C: Into<JRef<jclass>>> From<(C, &str, &str)> for JRef<jfieldID> {
    fn from(value: (C, &str, &str)) -> Self {
        let env = attach_current_thread();

        let name = CString::new(value.1).unwrap();
        let signature = CString::new(value.2).unwrap();
        JRef(jcall!(*env, GetFieldID, value.0.into().0, name.as_ptr(), signature.as_ptr()))
    }
}

impl<C: Into<JRef<jclass>>> From<(C, &str, &str, ())> for JRef<jfieldID> {
    fn from(value: (C, &str, &str, ())) -> Self {
        let env = attach_current_thread();

        let name = CString::new(value.1).unwrap();
        let signature = CString::new(value.2).unwrap();
        JRef(jcall!(
            *env,
            GetStaticFieldID,
            value.0.into().0,
            name.as_ptr(),
            signature.as_ptr()
        ))
    }
}

impl<C: Into<JRef<jclass>>, F: Into<JRef<jfieldID>>> From<(C, F, ())> for JRef<jobject> {
    fn from(value: (C, F, ())) -> Self {
        let env = attach_current_thread();

        JRef(jcall!(*env, GetStaticObjectField, value.0.into().0, value.1.into().0))
    }
}

impl<C: Into<JRef<jobject>>, F: Into<JRef<jfieldID>>> From<(C, F)> for JRef<jobject> {
    fn from(value: (C, F)) -> Self {
        let env = attach_current_thread();

        JRef(jcall!(*env, GetObjectField, value.0.into().0, value.1.into().0))
    }
}

enum LazyState<T: JValue, F: FnOnce() -> JRef<T>> {
    Uninitialized(F),
    Initializing,
    Initialized(T),
}

struct LazyStateHolder<T: JValue, F: FnOnce() -> JRef<T>> {
    state: LazyState<T, F>,
}

pub struct LazyJRef<T: JValue, F: FnOnce() -> JRef<T> = fn() -> JRef<T>> {
    holder: Mutex<LazyStateHolder<T, F>>,
}

unsafe impl<T: JValue, F: FnOnce() -> JRef<T>> Sync for LazyJRef<T, F> {}

unsafe impl<T: JValue, F: FnOnce() -> JRef<T>> Send for LazyJRef<T, F> {}

impl<T: JValue, F: FnOnce() -> JRef<T>> LazyJRef<T, F> {
    pub const fn new(into: F) -> Self {
        Self {
            holder: Mutex::new(LazyStateHolder {
                state: LazyState::Uninitialized(into),
            }),
        }
    }

    pub fn get(&self) -> JRef<T> {
        let state = &mut self.holder.lock().unwrap().state;

        match state {
            LazyState::Uninitialized(_) => {
                let initializer = mem::replace(state, LazyState::Initializing);

                let resolver = if let LazyState::Uninitialized(into) = initializer {
                    into
                } else {
                    panic!("unexpected state")
                };

                let obj = resolver().0.to_global_ref();

                *state = LazyState::Initialized(obj);

                JRef(obj)
            }
            LazyState::Initializing => {
                panic!("unexpected initializing")
            }
            LazyState::Initialized(obj) => JRef(*obj),
        }
    }
}

impl<T: JValue, F: FnOnce() -> JRef<T>> From<&LazyJRef<T, F>> for JRef<T> {
    fn from(value: &LazyJRef<T, F>) -> Self {
        value.get()
    }
}

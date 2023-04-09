use std::ops::Deref;

use jni_sys::jobject;

use crate::helper::{call::jcall, vm::attach_current_thread};

pub struct GlobalRef {
    value: jobject,
}

unsafe impl Sync for GlobalRef {}

unsafe impl Send for GlobalRef {}

impl Drop for GlobalRef {
    fn drop(&mut self) {
        let env = attach_current_thread();

        jcall!(*env, DeleteGlobalRef, self.value)
    }
}

impl Deref for GlobalRef {
    type Target = jobject;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl GlobalRef {
    pub fn new(obj: jobject) -> Self {
        let env = attach_current_thread();

        Self {
            value: jcall!(*env, NewGlobalRef, obj),
        }
    }
}

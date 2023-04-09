use error::Error;
use std::{error, ptr::null_mut};

use jni_sys::{jclass, jmethodID, jthrowable, JNIEnv};

use crate::helper::{
    call::jcall,
    lazy::{JRef, LazyJRef},
    strings::string_to_java_string,
};

pub struct SuppressedException {
    env: *mut JNIEnv,
    throwable: jthrowable,
}

impl SuppressedException {
    pub fn suppress(env: *mut JNIEnv) -> Self {
        let throwable = jcall!(env, ExceptionOccurred);

        if throwable != null_mut() {
            jcall!(env, ExceptionClear);
        }

        Self { env, throwable }
    }
}

impl Drop for SuppressedException {
    fn drop(&mut self) {
        if self.throwable != null_mut() {
            jcall!(self.env, Throw, self.throwable);
        }
    }
}

static C_IO_EXCEPTION: LazyJRef<jclass> = LazyJRef::new(|| JRef::from("java/io/IOException"));
static M_NEW_IOEXCEPTION: LazyJRef<jmethodID> =
    LazyJRef::new(|| JRef::from((&C_IO_EXCEPTION, "<init>", "(Ljava/lang/String;)V")));

pub fn rethrow_java_io_exception<R, F: FnOnce() -> Result<R, Box<dyn Error>>>(env: *mut JNIEnv, block: F) -> Option<R> {
    match block() {
        Ok(r) => Some(r),
        Err(err) => {
            let exception = jcall!(
                env,
                NewObject,
                C_IO_EXCEPTION.get().0,
                M_NEW_IOEXCEPTION.get().0,
                string_to_java_string(env, &err.to_string())
            );

            jcall!(env, Throw, exception);

            None
        }
    }
}

use error::Error;
use std::error;

use jni_sys::{jclass, jmethodID, JNIEnv};

use crate::helper::{
    call::jcall,
    lazy::{JRef, LazyJRef},
    strings::string_to_java_string,
};

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

use std::iter;

use jni_sys::{jbyteArray, jobject, jobjectArray, JNIEnv};

use crate::helper::call::jcall;

pub fn collect_java_bytes(env: *mut JNIEnv, array: jbyteArray) -> Vec<u8> {
    let length = jcall!(env, GetArrayLength, array);
    let mut ret = vec![0u8; length as usize];

    jcall!(env, GetByteArrayRegion, array, 0, length, ret.as_mut_ptr().cast());

    ret
}

pub fn iterate_object_array(env: *mut JNIEnv, array: jobjectArray) -> impl Iterator<Item = jobject> {
    let mut index = 0;
    let length = jcall!(env, GetArrayLength, array);

    iter::from_fn(move || {
        if index < length {
            let element = jcall!(env, GetObjectArrayElement, array, index);

            index += 1;

            Some(element)
        } else {
            None
        }
    })
}

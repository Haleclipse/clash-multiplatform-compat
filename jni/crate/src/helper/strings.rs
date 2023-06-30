use jni_sys::{jsize, jstring, JNIEnv};
use std::ptr::null_mut;

use crate::{helper::call::jcall, utils::scoped::Scoped};

pub fn java_string_to_string(env: *mut JNIEnv, string: jstring) -> String {
    let length = jcall!(env, GetStringLength, string);
    let ptr = Scoped::new(jcall!(env, GetStringChars, string, null_mut()), |c| {
        jcall!(env, ReleaseStringChars, string, *c)
    });

    let slice = unsafe { std::slice::from_raw_parts(*ptr, length as usize) };
    String::from_utf16(slice).expect("invalid UTF-16 string")
}

pub fn string_to_java_string(env: *mut JNIEnv, string: &str) -> jstring {
    let utf16_chars = string.encode_utf16().collect::<Vec<_>>();

    jcall!(env, NewString, utf16_chars.as_ptr(), utf16_chars.len() as jsize)
}

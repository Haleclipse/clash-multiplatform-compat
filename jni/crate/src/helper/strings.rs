use jni_sys::{jbyteArray, jclass, jint, jmethodID, jobject, jstring, JNIEnv};

use crate::helper::{
    array::collect_java_bytes,
    call::jcall,
    lazy::{JRef, LazyJRef},
    throwable::SuppressedException,
};

static C_STRING: LazyJRef<jclass> = LazyJRef::new(|| JRef::from("java/lang/String"));

static O_STANDARD_CHARSETS_UTF_8: LazyJRef<jobject> = LazyJRef::new(|| {
    JRef::from((
        "java/nio/charset/StandardCharsets",
        ("java/nio/charset/StandardCharsets", "UTF_8", "Ljava/nio/charset/Charset;", ()),
        (),
    ))
});

static M_GET_BYTES: LazyJRef<jmethodID> = LazyJRef::new(|| JRef::from((&C_STRING, "getBytes", "(Ljava/nio/charset/Charset;)[B")));

pub fn java_string_to_string(env: *mut JNIEnv, string: jstring) -> String {
    let _throwable = SuppressedException::suppress(env);

    let bytes = jcall!(
        env,
        CallObjectMethod,
        string,
        *M_GET_BYTES.get(),
        *O_STANDARD_CHARSETS_UTF_8.get()
    );

    String::from_utf8(collect_java_bytes(env, bytes as jbyteArray)).expect("invalid UTF-8 string")
}

static M_NEW_STRING: LazyJRef<jmethodID> = LazyJRef::new(|| JRef::from((&C_STRING, "<init>", "([BLjava/nio/charset/Charset;)V")));

pub fn string_to_java_string(env: *mut JNIEnv, string: &str) -> jstring {
    let _throwable = SuppressedException::suppress(env);

    let bytes = string.as_bytes();
    let java_bytes = jcall!(env, NewByteArray, bytes.len() as jint);

    jcall!(
        env,
        SetByteArrayRegion,
        java_bytes,
        0,
        bytes.len() as jint,
        bytes.as_ptr().cast()
    );

    jcall!(
        env,
        NewObject,
        *C_STRING.get(),
        *M_NEW_STRING.get(),
        java_bytes,
        *O_STANDARD_CHARSETS_UTF_8.get()
    ) as jstring
}

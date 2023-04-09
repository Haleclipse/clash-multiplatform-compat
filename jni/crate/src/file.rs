use jni_sys::{jboolean, jclass, jfieldID, jint, jlong, jmethodID, jobject, JNIEnv, JNI_FALSE};

use crate::{
    common::file::FileDescriptor,
    helper::{
        call::jcall,
        lazy::{JRef, LazyJRef},
        throwable::rethrow_java_io_exception,
    },
};

static F_SOCKET_CHANNEL_FD: LazyJRef<jfieldID> =
    LazyJRef::new(|| JRef::from(("sun/nio/ch/SocketChannelImpl", "fd", "Ljava/io/FileDescriptor;")));

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeGetFileDescriptorFromSocketChannel(
    env: *mut JNIEnv,
    _: jclass,
    channel: jobject,
) -> jobject {
    jcall!(env, GetObjectField, channel, *F_SOCKET_CHANNEL_FD.get())
}

static F_FILE_DESCRIPTOR_FD: LazyJRef<jfieldID> = LazyJRef::new(|| JRef::from(("java/io/FileDescriptor", "fd", "I")));
static F_FILE_DESCRIPTOR_HANDLE: LazyJRef<jfieldID> = LazyJRef::new(|| JRef::from(("java/io/FileDescriptor", "handle", "J")));

pub fn get_file_descriptor(env: *mut JNIEnv, fd: jobject) -> FileDescriptor {
    let handle = jcall!(env, GetLongField, fd, *F_FILE_DESCRIPTOR_HANDLE.get());
    if handle > 0 {
        handle as FileDescriptor
    } else {
        jcall!(env, GetIntField, fd, *F_FILE_DESCRIPTOR_FD.get()) as FileDescriptor
    }
}

fn set_file_descriptor(env: *mut JNIEnv, fd: jobject, value: FileDescriptor, is_socket: bool) {
    #[cfg(windows)]
    if is_socket {
        jcall!(env, SetIntField, fd, *F_FILE_DESCRIPTOR_FD.get(), value as jint);
    } else {
        jcall!(env, SetLongField, fd, *F_FILE_DESCRIPTOR_HANDLE.get(), value as jlong);
    }

    #[cfg(target_os = "linux")]
    {
        let _ = is_socket;

        jcall!(env, SetIntField, fd, *F_FILE_DESCRIPTOR_FD.get(), value as jint);
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeGetFileDescriptorHandle(
    env: *mut JNIEnv,
    _: jclass,
    fd: jobject,
) -> jlong {
    get_file_descriptor(env, fd) as jlong
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeSetFileDescriptorInheritable(
    env: *mut JNIEnv,
    _: jclass,
    fd: jobject,
    inheritable: jboolean,
) {
    let fd = get_file_descriptor(env, fd);

    rethrow_java_io_exception(env, || {
        #[cfg(windows)]
        crate::win32::file::set_file_descriptor_inheritable(fd, inheritable != JNI_FALSE)?;

        #[cfg(target_os = "linux")]
        crate::linux::file::set_file_descriptor_inheritable(fd, inheritable != JNI_FALSE)?;

        Ok(())
    });
}

static M_FILE_DESCRIPTOR_CLOSE: LazyJRef<jmethodID> = LazyJRef::new(|| JRef::from(("java/io/FileDescriptor", "close", "()V")));

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeCloseFileDescriptor(
    env: *mut JNIEnv,
    _: jclass,
    fd: jobject,
) {
    jcall!(env, CallVoidMethod, fd, *M_FILE_DESCRIPTOR_CLOSE.get())
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeCreatePipe(
    env: *mut JNIEnv,
    _: jclass,
    reader_fd: jobject,
    writer_fd: jobject,
) {
    rethrow_java_io_exception(env, || {
        #[cfg(windows)]
        let (reader, writer) = crate::win32::file::create_pipe()?;

        #[cfg(target_os = "linux")]
        let (reader, writer) = crate::linux::file::create_pipe()?;

        set_file_descriptor(env, reader_fd, reader, false);
        set_file_descriptor(env, writer_fd, writer, false);

        Ok(())
    });
}

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeCreateUnixSocketPair(
    env: *mut JNIEnv,
    _: jclass,
    first_fd: jobject,
    second_fd: jobject,
) {
    rethrow_java_io_exception(env, || {
        #[cfg(windows)]
        let (first, second) = crate::win32::file::create_socket_pair()?;

        #[cfg(target_os = "linux")]
        let (first, second) = crate::linux::file::create_socket_pair()?;

        set_file_descriptor(env, first_fd, first, true);
        set_file_descriptor(env, second_fd, second, true);

        Ok(())
    });
}

static C_SOCKET_CHANNEL_IMPL: LazyJRef<jclass> = LazyJRef::new(|| JRef::from("sun/nio/ch/SocketChannelImpl"));
static M_NEW_SOCKET_CHANNEL_IMPL: LazyJRef<jmethodID> = LazyJRef::new(|| {
    JRef::from((
        &C_SOCKET_CHANNEL_IMPL,
        "<init>",
        "(Ljava/nio/channels/spi/SelectorProvider;Ljava/net/ProtocolFamily;Ljava/io/FileDescriptor;Ljava/net/SocketAddress;)V",
    ))
});

#[no_mangle]
pub extern "C" fn Java_com_github_kr328_clash_compat_FileCompat_nativeNewSocketChannel(
    env: *mut JNIEnv,
    _: jclass,
    sp: jobject,
    family: jobject,
    fd: jobject,
    address: jobject,
) -> jobject {
    jcall!(
        env,
        NewObject,
        *C_SOCKET_CHANNEL_IMPL.get(),
        *M_NEW_SOCKET_CHANNEL_IMPL.get(),
        sp,
        family,
        fd,
        address
    )
}

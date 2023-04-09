use errno::{errno, set_errno, Errno};

pub fn syscall<R, F: FnOnce() -> R>(func: F) -> Result<R, Errno> {
    set_errno(Errno(0));

    let r = func();
    let errno = errno();
    if errno.0 != 0 {
        return Err(errno);
    }

    Ok(r)
}

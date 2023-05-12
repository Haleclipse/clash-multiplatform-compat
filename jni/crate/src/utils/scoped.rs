use std::{
    mem,
    ops::{Deref, DerefMut},
};

pub struct Scoped<T, C: FnOnce(&T) = fn(&T)> {
    pub value: T,
    closer: Option<C>,
}

impl<T, C: FnOnce(&T)> Scoped<T, C> {
    pub fn new(initial: T, closer: C) -> Self {
        Scoped {
            value: initial,
            closer: Some(closer),
        }
    }

    pub fn swap(&mut self, new: T) -> T {
        mem::replace(&mut self.value, new)
    }
}

impl<T, C: FnOnce(&T)> Deref for Scoped<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, C: FnOnce(&T)> DerefMut for Scoped<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, C: FnOnce(&T)> Drop for Scoped<T, C> {
    fn drop(&mut self) {
        if let Some(closer) = mem::replace(&mut self.closer, None) {
            closer(&self.value)
        }
    }
}

#[cfg(target_os = "linux")]
impl Scoped<std::ffi::c_int, fn(&std::ffi::c_int)> {
    fn close_fd(fd: &std::ffi::c_int) {
        unsafe {
            libc::close(*fd);
        }
    }

    pub fn new_fd(fd: std::ffi::c_int) -> Self {
        Self::new(fd, Scoped::close_fd)
    }
}

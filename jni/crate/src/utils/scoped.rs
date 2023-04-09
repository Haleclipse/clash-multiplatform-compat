use std::{
    mem,
    ops::{Deref, DerefMut},
};

pub struct Scoped<T, C: FnMut(&T)> {
    pub value: T,
    closer: Box<C>,
}

impl<T, C: FnMut(&T)> Scoped<T, C> {
    pub fn new(initial: T, closer: C) -> Self {
        Scoped {
            value: initial,
            closer: Box::new(closer),
        }
    }

    pub fn swap(&mut self, new: T) -> T {
        mem::replace(&mut self.value, new)
    }
}

impl<T, C: FnMut(&T)> Deref for Scoped<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, C: FnMut(&T)> DerefMut for Scoped<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T, C: FnMut(&T)> Drop for Scoped<T, C> {
    fn drop(&mut self) {
        (self.closer)(&self.value);
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

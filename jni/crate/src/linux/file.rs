use std::{error::Error, os::fd::RawFd};

use libc::{fcntl, pipe2, socketpair, AF_UNIX, FD_CLOEXEC, F_GETFD, F_SETFD, O_CLOEXEC, SOCK_STREAM};

use crate::{common::file::FileDescriptor, linux::errno::syscall, utils::scoped::Scoped};

pub fn set_file_descriptor_inheritable(fd: FileDescriptor, inheritable: bool) -> Result<(), Box<dyn Error>> {
    unsafe {
        let mut flags = syscall(|| fcntl(fd as RawFd, F_GETFD))?;

        if inheritable {
            flags &= !FD_CLOEXEC;
        } else {
            flags |= FD_CLOEXEC;
        }

        syscall(|| fcntl(fd as RawFd, F_SETFD, flags & !FD_CLOEXEC))?;
    }

    Ok(())
}

pub fn create_socket_pair() -> Result<(FileDescriptor, FileDescriptor), Box<dyn Error>> {
    let mut pair = [-1; 2];

    unsafe {
        syscall(|| socketpair(AF_UNIX, SOCK_STREAM, 0, pair.as_mut_ptr()))?;
    }

    let mut first = Scoped::new_fd(pair[0]);
    let mut second = Scoped::new_fd(pair[1]);

    set_file_descriptor_inheritable(*first as FileDescriptor, false)?;
    set_file_descriptor_inheritable(*second as FileDescriptor, false)?;

    Ok((first.swap(-1) as FileDescriptor, second.swap(-1) as FileDescriptor))
}

pub fn create_pipe() -> Result<(FileDescriptor, FileDescriptor), Box<dyn Error>> {
    let mut pipe: [i32; 2] = Default::default();

    unsafe {
        syscall(|| pipe2(pipe.as_mut_ptr(), O_CLOEXEC))?;
    }

    let mut rx = Scoped::new_fd(pipe[0]);
    let mut tx = Scoped::new_fd(pipe[1]);

    set_file_descriptor_inheritable(*rx as FileDescriptor, false)?;
    set_file_descriptor_inheritable(*tx as FileDescriptor, false)?;

    Ok((rx.swap(-1) as FileDescriptor, tx.swap(-1) as FileDescriptor))
}
